use oj::api_analysis;

use actix_web::http::header::TryIntoHeaderValue;
use actix_web::{
    get, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder,
};
use console::style;
use reqwest;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::{to_string_pretty, Value};
use std::io::{BufReader, Error, ErrorKind};
use std::{fs, io};


/*************************************************************************
【函数名称】                post_users
【函数功能】                注册一名新用户，目的是方便前端的测试
【参数】                   user_info用户信息，其中id为None
【返回值】                 Response，解析后可判断是否新增成功（重名则失败）
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
async fn post_users(user_info: api_analysis::APIPostUsers) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .post("http://127.0.0.1:12345/users")
        .json(&user_info)
        .send()
        .await
        .unwrap()
}

/*************************************************************************
【函数名称】                get_contest_rank
【函数功能】                获取目前的用户排行榜，具体实现依赖于后端，这里只负责获取
【参数】                   contest_id目标比赛的编号
【返回值】                 Result<String>，如果获取失败返回Err，否则为获取的信息
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
async fn get_contest_rank(contest_id: usize) -> std::io::Result<String> {
    let body = reqwest::get(
        "http://127.0.0.1:12345/contests/".to_string() + &contest_id.to_string() + "/ranklist",
    )
    .await
    .unwrap()
    .text()
    .await
    .unwrap();
    Ok(body)
}

/*************************************************************************
【函数名称】                get_work
【函数功能】                获取提交的任务的评测信息，main会多次调用此函数
                         直至status为finished
【参数】                   work_id需要获取的任务的编号
【返回值】                 Result<String>，如果获取失败返回Err，否则为获取的信息
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
async fn get_work(work_id: usize) -> std::io::Result<String> {
    let body = reqwest::get("http://127.0.0.1:12345/jobs/".to_string() + &work_id.to_string())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    Ok(body)
}

/*************************************************************************
【函数名称】                post_work
【函数功能】                将获取的任务提交到后端，等待其响应，获取所提交任务的编号
【参数】                   info为已经打包好的提交任务
【返回值】                 Response，为后端返回的信息，目前非阻塞评测下状态为waiting
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
async fn post_work(info: api_analysis::APIPostJob) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .post("http://127.0.0.1:12345/jobs")
        .json(&info)
        .send()
        .await
        .unwrap()
}

/*************************************************************************
【函数名称】                command
【函数功能】                获取任务指令序号(1/2/3/4)，选择相应指令，并给出对应输出
                          1号指令提交单个代码并以较友好格式输出，2号指令连续获取多个
                          文件并且以json形式返回所有结果，3号指令获取排行榜并以表格
                          形式打印出来，4号指令获取用户名并为其注册为一个新用户
【参数】                   无
【返回值】                 Result<()>，如果出现异常，则Err退出，否则Ok退出
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
async fn command() -> std::io::Result<()> {
    println!("1.提交单个代码\n2.从多文件中批量提交\n3.查看比赛排行榜\n4.新增用户\n请输入(1/2/3/4):");
    let mut pattern_choose = String::new();
    io::stdin().read_line(&mut pattern_choose)?;
    if pattern_choose.trim() == "1" {
        println!("File Name:");
        let mut file_name = String::new();
        io::stdin().read_line(&mut file_name)?;
        let open_file = BufReader::new(fs::File::open(file_name.trim())?);
        let mut body: api_analysis::APIPostJob = serde_json::from_reader(open_file)?;
        let source_file_name = body.source_code.clone();
        body.source_code = fs::read_to_string(source_file_name)?;
        let response = post_work(body).await;
        let json_response: api_analysis::APIPostResponse = response.json().await.expect(
            format!("case ? incorrect: cannot decode response body as JSON, status code is ?",)
                .as_str(),
        );
        println!("{}", style("Waiting").cyan());
        loop {
            let final_response = get_work(json_response.id).await?;
            let final_json_response: api_analysis::APIPostResponse =
                serde_json::from_str(&final_response)?;
            if final_json_response.result != api_analysis::APIPostResponseResult::Running {
                //tui
                println!("{}", style("Running").blue());
                std::thread::sleep(std::time::Duration::from_secs(1));
                let case_nums = final_json_response.cases.len()-1;
                let average_value = 100.0 / (case_nums as f32);
                println!("case0:{}",api_analysis::APIPostResponseResult::show_str(
                    &final_json_response.cases[0].result));
                for num1 in 1..=case_nums {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    for num2 in 1..=num1 {
                        if final_json_response.cases[num2].result
                            == api_analysis::APIPostResponseResult::Accepted
                        {
                            print!(
                                "case{}:{:.2} {} ",
                                num2,
                                average_value,
                                api_analysis::APIPostResponseResult::show_str(
                                    &final_json_response.cases[num2].result
                                )
                            );
                        } else {
                            print!(
                                "case{}:{:.2} {} ",
                                num2,
                                0.0,
                                api_analysis::APIPostResponseResult::show_str(
                                    &final_json_response.cases[num2].result
                                )
                            );
                        }
                    }
                    println!();
                }
                println!("{}", style("Finished").green());
                println!(
                    "Scores:{:.2} {}",
                    final_json_response.score,
                    api_analysis::APIPostResponseResult::show_str(&final_json_response.result)
                );
                //tui
                break;
            }
        }
    } else if pattern_choose.trim() == "2" {
        let mut file_name_vec = Vec::new();
        let mut json_vec = Vec::new();
        loop {
            println!("File Name:");
            let mut file_name = String::new();
            io::stdin().read_line(&mut file_name)?;
            if file_name.trim() != "".to_string() {
                file_name_vec.push(file_name.trim().to_string());
            } else {
                break;
            }
        }
        for file_name in file_name_vec {
            let open_file = BufReader::new(fs::File::open(file_name)?);
            let mut body: api_analysis::APIPostJob = serde_json::from_reader(open_file)?;
            let source_file_name = body.source_code.clone();
            body.source_code = fs::read_to_string(source_file_name)?;
            let response = post_work(body).await;
            let json_response: api_analysis::APIPostResponse = response.json().await.expect(
                format!("case ? incorrect: cannot decode response body as JSON, status code is ?",)
                    .as_str(),
            );
            loop {
                let final_response = get_work(json_response.id).await?;
                let final_json_response: api_analysis::APIPostResponse =
                    serde_json::from_str(&final_response)?;
                if final_json_response.result != api_analysis::APIPostResponseResult::Running {
                    json_vec.push(final_json_response);
                    break;
                }
            }
        }
        println!("{}", to_string_pretty(&json_vec)?);
    } else if pattern_choose.trim() == "3" {
        println!("Contest ID:");
        let mut user_id_str = String::new();
        io::stdin().read_line(&mut user_id_str)?;
        let user_id = user_id_str.trim().parse().unwrap_or(1000000);
        let result = get_contest_rank(user_id).await?;
        let rank_list: Vec<api_analysis::UserRankResponse> = serde_json::from_str(result.as_str())?;
        println!(
            "{: >3}  {: <3}  {: <10}  {}",
            style("Rank").blue().bold(),
            style("ID").magenta().bold(),
            style("Name").cyan().bold(),
            "Scores"
        );
        for single_rank in rank_list {
            println!(
                "{:0>3}   {:0>3}  {: <10}  {:?}",
                style(single_rank.rank).blue().bold(),
                style(single_rank.user.id.unwrap()).magenta().bold(),
                style(single_rank.user.name).cyan().bold(),
                single_rank.scores
            );
        }
    } else if pattern_choose.trim() == "4"{
        println!("User Name:");
        let mut user_id_str = String::new();
        io::stdin().read_line(&mut user_id_str)?;
        let response=post_users(
            api_analysis::APIPostUsers{id:None,name:user_id_str.trim().to_string()})
            .await;
        let response_json:reqwest::Result<api_analysis::APIPostUsers>=response.json().await;
        if let Err(_e)=response_json{
            return Err(Error::new(ErrorKind::Other, "用户重名！"));
        }
    } else {
        return Err(Error::new(ErrorKind::Other, "输入选项(1/2/3/4)错误！"));
    }
    Ok(())
}

/*************************************************************************
【函数名称】                main
【函数功能】                循环调用command函数，并且在每次调用后询问是否退出
【参数】                   无
【返回值】                 Result<()>，如果出现异常，则Err退出
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    loop {
        match command().await {
            Ok(_o) => {
                println!("操作成功！");
                println!("继续？(Y/N):");
                let mut whether_continue = String::new();
                std::io::stdin().read_line(&mut whether_continue)?;
                if whether_continue.trim().to_lowercase() != "y" {
                    break;
                }
            }
            Err(e) => {
                println!("操作失败，失败原因：{}", e.to_string());
                println!("继续？(Y/N):");
                let mut whether_continue = String::new();
                std::io::stdin().read_line(&mut whether_continue)?;
                if whether_continue.trim().to_lowercase() != "y" {
                    break;
                }
            }
        }
    }
    Ok(())
}
