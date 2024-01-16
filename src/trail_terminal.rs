use crate::config_analysis;
use crate::API_ARRAY;
use crate::CONTEST_HIGHEST_USER_SUBMIT_ARRAY;
use crate::RECORD_USER_PROBLEM_ARRAY;
use crate::{
    api_analysis, ALL_USER_SUBMIT_ARRAY, CONTEST_ALL_USER_SUBMIT_ARRAY, CONTEST_ARRAY,
    HIGHEST_USER_SUBMIT_ARRAY, PROBLEM_ID_ARRAY, QUEUEING_RESPONSE_ARRAY,
    RECORD_CONTEST_USER_PROBLEM_ARRAY, RESPONSE_ARRAY, USER_ARRAY, USER_ID_ARRAY,
};
use actix_web::{
    get, main, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder,
};
use chrono;
use env_logger;
use log;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
use std::process::{Command, Stdio};
use std::string::String;
use std::sync::Mutex;
use std::{env, io};
use wait4;
use wait4::Wait4;
use wait_timeout::ChildExt;

//作为后台来处理post_jobs发送过来的数据，为了简化main.rs的部分函数而设置
/*************************************************************************
【函数名称】                api_terminal
【函数功能】                处理job的相应信息，对每个测试点做出测试
【参数】                   config:配置，api_body:提交的作答信息
【返回值】                 Result类型，Ok中附带Response所需的json形式响应
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub fn api_terminal(
    config_body: config_analysis::Config,
    api_body: api_analysis::APIPostJob,
) -> io::Result<api_analysis::APIPostResponse> {
    // println!("In the start now!");
    let mut response_body = api_analysis::APIPostResponse::new();
    response_body.submission = api_body.clone();

    let mut relevant_problem = config_analysis::ConfigProblem::new();
    let mut problem_contain = false;
    for pro in config_body.problems {
        if pro.id == api_body.problem_id {
            relevant_problem = pro.clone();
            problem_contain = true;
            break;
        }
    }
    if !problem_contain {
        return Err(Error::new(ErrorKind::Other, "No Problem Found"));
    }
    let tmp_num = QUEUEING_RESPONSE_ARRAY.lock().unwrap().len();
    let tmp = "tmp".to_string() + &tmp_num.to_string();
    let mut command_args: Vec<String> = Vec::new();
    let mut language_find = false;
    let mut command_file_name = String::new();
    for relevant_language in config_body.languages {
        if relevant_language.name == api_body.language {
            language_find = true;
            command_args = relevant_language.command.clone();
            command_file_name = tmp.clone() + "/" + &relevant_language.file_name.clone();
            break;
        }
    }
    if !language_find {
        return Err(Error::new(ErrorKind::Other, " No same language!"));
    }
    fs::create_dir_all(tmp.clone())?;
    fs::File::create(&command_file_name)?;
    fs::write(&command_file_name, api_body.source_code)?;
    command_args.reverse();
    let mut command_new_arg = String::new();
    match command_args.pop() {
        Some(s) => command_new_arg = s,
        None => return Err(Error::new(ErrorKind::Other, "the command is empty")),
    }
    command_args.reverse();
    for command_num in 0..command_args.len() {
        if command_args[command_num] == "%INPUT%" {
            command_args[command_num] = command_file_name.clone();
        } else if command_args[command_num] == "%OUTPUT%" {
            command_args[command_num] = (tmp.clone() + "/test.exe").to_string();
        }
    }
    // println!("In the terminal now!");
    // println!("{}->{:?}", command_new_arg.clone(), command_args.clone());
    match Command::new(command_new_arg)
        .args(&mut command_args)
        .status()
    {
        Ok(o) => {
            if !o.success() {
                /*************************************************************************
                【模块功能】               处理程序运行失败的情况，直接返回CompilationError，
                                         跳过编译失败下的测试点
                【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                【更改记录】                2022-9-8 由林奕辰增加注释
                *************************************************************************/
                // println!("command fail!");
                // println!("{}", o.clone().to_string());
                response_body.result = api_analysis::APIPostResponseResult::CompilationError;

                let mut single_case_data = api_analysis::APIPostResponseCase::new();
                single_case_data.result = api_analysis::APIPostResponseResult::CompilationError;
                single_case_data.id = 0;
                response_body.cases.push(single_case_data);

                fs::remove_dir_all(tmp.clone())?;
                response_body.updated_time = chrono::prelude::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                    .to_string();
                for _i in 0..relevant_problem.cases.len() {
                    let mut another_case_data = api_analysis::APIPostResponseCase::new();
                    another_case_data.id = response_body.cases.len();
                    response_body.cases.push(another_case_data);
                }
                response_body.result = api_analysis::APIPostResponseResult::CompilationError;
                response_body.state = api_analysis::APIPostResponseState::Finished;
                {
                    let response_clone_for_record = response_body.clone();
                    let mut find_record = false;
                    if let Ok(mut ok) = RECORD_USER_PROBLEM_ARRAY.lock() {
                        for record_num in 0..ok.len() {
                            if ok[record_num].user_id
                                == response_clone_for_record.submission.user_id
                                && ok[record_num].problem_id
                                    == response_clone_for_record.submission.problem_id
                            {
                                ok[record_num]
                                    .scores
                                    .push(response_clone_for_record.score.clone());
                                find_record = true;
                                let mut highest_user_submit_add = true;
                                for passed_score in ok[record_num].scores.clone() {
                                    if response_clone_for_record.score <= passed_score {
                                        highest_user_submit_add = false;
                                        break;
                                    }
                                }
                                if highest_user_submit_add {
                                    HIGHEST_USER_SUBMIT_ARRAY
                                        .lock()
                                        .unwrap()
                                        .push(response_clone_for_record.submission.user_id.clone());
                                }
                                break;
                            }
                        }
                        if !find_record {
                            let mut new_record = api_analysis::RecordUserProblem::new();
                            new_record.scores = Vec::new();
                            new_record.scores.push(response_clone_for_record.score);
                            HIGHEST_USER_SUBMIT_ARRAY
                                .lock()
                                .unwrap()
                                .push(response_clone_for_record.submission.user_id.clone());
                            new_record.problem_id = response_clone_for_record.submission.problem_id;
                            new_record.user_id = response_clone_for_record.submission.user_id;
                            ok.push(new_record);
                        }
                        ALL_USER_SUBMIT_ARRAY
                            .lock()
                            .unwrap()
                            .push(response_clone_for_record.submission.user_id.clone());
                    }
                    let the_contest_id = response_body.submission.contest_id;
                    if the_contest_id > 0 {
                        if let Ok(mut ok) = RECORD_CONTEST_USER_PROBLEM_ARRAY.lock() {
                            for record_num in 0..ok.len() {
                                if ok[record_num].user_id
                                    == response_clone_for_record.submission.user_id
                                    && ok[record_num].problem_id
                                        == response_clone_for_record.submission.problem_id
                                    && ok[record_num].contest_id == the_contest_id
                                {
                                    ok[record_num]
                                        .scores
                                        .push(response_clone_for_record.score.clone());
                                    find_record = true;
                                    let mut highest_user_submit_add = true;
                                    for passed_score in ok[record_num].scores.clone() {
                                        if response_clone_for_record.score <= passed_score {
                                            highest_user_submit_add = false;
                                            break;
                                        }
                                    }
                                    if highest_user_submit_add {
                                        CONTEST_HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap().push((
                                            the_contest_id,
                                            response_clone_for_record.submission.user_id.clone(),
                                        ));
                                    }
                                    break;
                                }
                            }
                            if !find_record {
                                let mut new_record = api_analysis::RecordContestUserProblem::new();
                                new_record.scores = Vec::new();
                                new_record.contest_id =
                                    response_clone_for_record.submission.contest_id;
                                new_record.scores.push(response_clone_for_record.score);
                                HIGHEST_USER_SUBMIT_ARRAY
                                    .lock()
                                    .unwrap()
                                    .push(response_clone_for_record.submission.user_id.clone());
                                new_record.problem_id =
                                    response_clone_for_record.submission.problem_id;
                                new_record.user_id = response_clone_for_record.submission.user_id;
                                ok.push(new_record);
                            }
                            CONTEST_ALL_USER_SUBMIT_ARRAY.lock().unwrap().push((
                                the_contest_id,
                                response_clone_for_record.submission.user_id.clone(),
                            ));
                        }
                    }
                }
                return Ok(response_body);
            } else {
                // println!("in the command now!");
                let mut single_case_data = api_analysis::APIPostResponseCase::new();
                single_case_data.result = api_analysis::APIPostResponseResult::CompilationSuccess;
                single_case_data.id = 0;
                response_body.cases.push(single_case_data);
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    /*************************************************************************
    【模块功能】                处理程序运行成功的情况，对每个测试点进行运行处理，
                              case0的结果为CompilationSuccess
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
     *************************************************************************/
    // println!("In the cases now!");
    for single_case in &relevant_problem.cases {
        let start_running_time = chrono::prelude::Utc::now(); //remark the time to start
        if single_case.time_limit > 0 {
            // println!("time_limit:{}", single_case.time_limit);
            let set_timeout = std::time::Duration::from_micros(single_case.time_limit);
            let mut child = Command::new(tmp.clone() + "/test.exe")
                .stdin(Stdio::from(fs::File::open(&single_case.input_file)?))
                .stdout(Stdio::from(fs::File::create(tmp.clone() + "/out.txt")?))
                .stderr(Stdio::null())
                .spawn()?;
            //let status_code=child.wait_timeout(set_timeout)?;
            match child.wait_timeout(set_timeout) {
                Ok(o) => {
                    match o {
                        Some(s) => {
                            if !s.success() {
                                /*************************************************************************
                                【模块功能】                单个测试点时间异常，将该测试点置为RuntimeError，如果为第一个
                                                         出现错误的测试点，job的result为RuntimeError
                                【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                                【更改记录】                2022-9-8 由林奕辰增加注释
                                 *************************************************************************/
                                if response_body.result
                                    == api_analysis::APIPostResponseResult::Waiting
                                {
                                    response_body.result =
                                        api_analysis::APIPostResponseResult::RuntimeError;
                                }

                                let end_running_time = chrono::prelude::Utc::now(); //remark the time to end
                                let mut single_case_data = api_analysis::APIPostResponseCase::new();
                                single_case_data.result =
                                    api_analysis::APIPostResponseResult::RuntimeError;
                                single_case_data.id = response_body.cases.len();
                                single_case_data.memory = 0;
                                if let Some(ok) = end_running_time
                                    .signed_duration_since(start_running_time)
                                    .num_microseconds()
                                {
                                    single_case_data.time = ok as usize;
                                }
                                response_body.cases.push(single_case_data);
                                continue;
                            }
                        }
                        None => {
                            /*************************************************************************
                            【模块功能】                判断测试点运行超时，将该测试点置为TimeLimitExceeded，如果为第一个
                                                     出现错误的测试点，job的result为TimeLimitExceeded
                            【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                            【更改记录】                2022-9-8 由林奕辰增加注释
                             *************************************************************************/
                            if response_body.result == api_analysis::APIPostResponseResult::Waiting
                            {
                                response_body.result =
                                    api_analysis::APIPostResponseResult::TimeLimitExceeded;
                            }

                            let end_running_time = chrono::prelude::Utc::now(); //remark the time to end
                            let mut single_case_data = api_analysis::APIPostResponseCase::new();
                            single_case_data.result =
                                api_analysis::APIPostResponseResult::TimeLimitExceeded;
                            single_case_data.id = response_body.cases.len();
                            single_case_data.memory = 0;
                            if let Some(ok) = end_running_time
                                .signed_duration_since(start_running_time)
                                .num_microseconds()
                            {
                                single_case_data.time = ok as usize;
                            }
                            response_body.cases.push(single_case_data);
                            child.kill()?;
                            child.wait()?;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    if response_body.result == api_analysis::APIPostResponseResult::Waiting {
                        response_body.result = api_analysis::APIPostResponseResult::RuntimeError;
                    }

                    let end_running_time = chrono::prelude::Utc::now(); //remark the time to end
                    let mut single_case_data = api_analysis::APIPostResponseCase::new();
                    single_case_data.result = api_analysis::APIPostResponseResult::RuntimeError;
                    single_case_data.id = response_body.cases.len();
                    single_case_data.memory = 0;
                    if let Some(ok) = end_running_time
                        .signed_duration_since(start_running_time)
                        .num_microseconds()
                    {
                        single_case_data.time = ok as usize;
                    }
                    response_body.cases.push(single_case_data);
                    continue;
                }
            }
            //println!("time limit decide end");
        } else {
            match Command::new(tmp.clone() + "/test.exe")
                .stdin(Stdio::from(fs::File::open(&single_case.input_file)?))
                .stdout(Stdio::from(fs::File::create(tmp.clone() + "/out.txt")?))
                .stderr(Stdio::null())
                .status()
            {
                Ok(o) => {
                    if !o.success() {
                        if response_body.result == api_analysis::APIPostResponseResult::Waiting {
                            response_body.result =
                                api_analysis::APIPostResponseResult::RuntimeError;
                        }

                        let end_running_time = chrono::prelude::Utc::now(); //remark the time to end
                        let mut single_case_data = api_analysis::APIPostResponseCase::new();
                        single_case_data.result = api_analysis::APIPostResponseResult::RuntimeError;
                        single_case_data.id = response_body.cases.len();
                        single_case_data.memory = 0;
                        if let Some(ok) = end_running_time
                            .signed_duration_since(start_running_time)
                            .num_microseconds()
                        {
                            single_case_data.time = ok as usize;
                        }
                        response_body.cases.push(single_case_data);
                        continue;
                    }
                }
                Err(e) => {
                    if response_body.result == api_analysis::APIPostResponseResult::Waiting {
                        response_body.result = api_analysis::APIPostResponseResult::RuntimeError;
                    }

                    let end_running_time = chrono::prelude::Utc::now(); //remark the time to end
                    let mut single_case_data = api_analysis::APIPostResponseCase::new();
                    single_case_data.result = api_analysis::APIPostResponseResult::RuntimeError;
                    single_case_data.id = response_body.cases.len();
                    single_case_data.memory = 0;
                    if let Some(ok) = end_running_time
                        .signed_duration_since(start_running_time)
                        .num_microseconds()
                    {
                        single_case_data.time = ok as usize;
                    }
                    response_body.cases.push(single_case_data);
                    continue;
                }
            }
        }
        let mut test_out = String::new();
        let mut standard_out = String::new();
        fs::File::open(tmp.clone() + "/out.txt")?.read_to_string(&mut test_out)?;
        fs::File::open(&single_case.answer_file)?.read_to_string(&mut standard_out)?;
        let end_running_time = chrono::prelude::Utc::now(); //remark the time to end

        //memory

        //spj
        if let Some(misc_data) = relevant_problem.clone().misc {
            if let Some(pack_pattern) = misc_data.special_judge {
                let mut pack_rev = pack_pattern.clone();
                pack_rev.reverse();
                let mut command1 = pack_rev.pop().unwrap();
                // if command1=="python3"{
                //     command1="python".to_string();
                // }
                pack_rev.reverse();
                for i in 0..pack_rev.len() {
                    if pack_rev[i] == "%OUTPUT%" {
                        pack_rev[i] = (tmp.clone() + "/out.txt").to_string();
                    } else if pack_rev[i] == "%ANSWER%" {
                        pack_rev[i] = single_case.answer_file.clone();
                    }
                }
                // for i in pack_rev.clone() {
                //     println!("{}", i);
                // }
                let spj_data = fs::File::create(tmp.clone() + "/spj.txt")?;
                match Command::new(command1)
                    .args(pack_rev.clone())
                    .stdout(Stdio::from(spj_data))
                    .status()
                {
                    Ok(o) => {
                        if !o.success() {
                            return Err(Error::new(ErrorKind::UnexpectedEof, "SPJ Error"));
                        }
                    }
                    Err(_e) => return Err(Error::new(ErrorKind::UnexpectedEof, "SPJ Error")),
                }
                // Command::new(command1).args(pack_rev.clone()).
                //     stdout(Stdio::from(spj_data)).status().unwrap();
                let mut spj_answer = BufReader::new(fs::File::open(tmp.clone() + "/spj.txt")?);
                let mut spj_result = String::new();
                let mut spj_info = String::new();
                spj_answer.read_line(&mut spj_result)?;
                spj_answer.read_line(&mut spj_info)?;
                // println!("spj_result:{}", spj_result.clone());
                // println!("spj_info:{}", spj_info.clone());
                let mut single_case_data = api_analysis::APIPostResponseCase::new();
                single_case_data.id = response_body.cases.len();
                single_case_data.memory = 0;
                if let Some(ok) = end_running_time
                    .signed_duration_since(start_running_time)
                    .num_microseconds()
                {
                    single_case_data.time = ok as usize;
                }
                if spj_result.trim_end() == "Accepted" {
                    response_body.score += single_case.score;
                    single_case_data.result = api_analysis::APIPostResponseResult::Accepted;
                } else {
                    if response_body.result == api_analysis::APIPostResponseResult::Waiting {
                        response_body.result = api_analysis::APIPostResponseResult::WrongAnswer;
                    }
                    single_case_data.result = api_analysis::APIPostResponseResult::WrongAnswer;
                }
                single_case_data.info = spj_info.trim_end().to_string();
                response_body.cases.push(single_case_data);
                continue;
            }
        }
        //spj

        /*************************************************************************
        【模块功能】                如果问题的type为standard，则进行标准化处理
        【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
        【更改记录】                2022-9-8 由林奕辰增加注释
         *************************************************************************/
        match relevant_problem.r#type {
            config_analysis::ConfigProblemType::Standard => {
                test_out = config_analysis::deal_string_standard(test_out);
                standard_out = config_analysis::deal_string_standard(standard_out);
            }
            _ => (),
        }
        /*************************************************************************
        【模块功能】                如果处理后的输出结果和答案结果相吻合，则该测试点通过，Accepted
        【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
        【更改记录】                2022-9-8 由林奕辰增加注释
         *************************************************************************/
        if test_out == standard_out {
            response_body.score += single_case.score;
            let mut single_case_data = api_analysis::APIPostResponseCase::new();
            single_case_data.result = api_analysis::APIPostResponseResult::Accepted;
            single_case_data.id = response_body.cases.len();
            single_case_data.memory = test_out.clone().len();
            if relevant_problem.cases[0].memory_limit > 0
                && single_case_data.memory > relevant_problem.cases[0].memory_limit as usize
            {
                single_case_data.result = api_analysis::APIPostResponseResult::MemoryLimitExceeded;
                if response_body.result == api_analysis::APIPostResponseResult::Waiting {
                    response_body.result = api_analysis::APIPostResponseResult::MemoryLimitExceeded;
                }
                response_body.score -= single_case.score;
            }
            if let Some(ok) = end_running_time
                .signed_duration_since(start_running_time)
                .num_microseconds()
            {
                single_case_data.time = ok as usize;
            }
            response_body.cases.push(single_case_data);
        } else {
            if response_body.result == api_analysis::APIPostResponseResult::Waiting {
                response_body.result = api_analysis::APIPostResponseResult::WrongAnswer;
            }

            let mut single_case_data = api_analysis::APIPostResponseCase::new();
            single_case_data.result = api_analysis::APIPostResponseResult::WrongAnswer;
            single_case_data.id = response_body.cases.len();
            single_case_data.memory = test_out.clone().len();
            if relevant_problem.cases[0].memory_limit > 0
                && single_case_data.memory > relevant_problem.cases[0].memory_limit as usize
            {
                single_case_data.result = api_analysis::APIPostResponseResult::MemoryLimitExceeded;
                response_body.result = api_analysis::APIPostResponseResult::MemoryLimitExceeded;
            }
            if let Some(ok) = end_running_time
                .signed_duration_since(start_running_time)
                .num_microseconds()
            {
                single_case_data.time = ok as usize;
            }
            response_body.cases.push(single_case_data);
        }
    }
    if let Some(misc_data) = relevant_problem.clone().misc {
        if let Some(pack_pattern) = misc_data.packing {
            let cases = response_body.cases.clone();
            for single_pack in pack_pattern {
                for num in 0..single_pack.len() {
                    if cases[single_pack[num]].result
                        != api_analysis::APIPostResponseResult::Accepted
                    {
                        for num1 in num + 1..single_pack.len() {
                            response_body.cases[single_pack[num1]].result =
                                api_analysis::APIPostResponseResult::Skipped;
                        }
                        break;
                    }
                }
            }
        }
    }
    // println!("In the end time now!");
    fs::remove_dir_all(tmp.clone())?;
    response_body.updated_time = chrono::prelude::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    response_body.state = api_analysis::APIPostResponseState::Finished;
    if response_body.result == api_analysis::APIPostResponseResult::Waiting {
        response_body.result = api_analysis::APIPostResponseResult::Accepted;
    }
    //println!("CASES::::{:?}",response_body.cases.clone());
    {
        /*************************************************************************
        【模块功能】               在得到数据后，进行存储处理，为了便于后续的排行榜处理
        【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
        【更改记录】                2022-9-8 由林奕辰增加注释
        *************************************************************************/
        let response_clone_for_record = response_body.clone();
        let mut find_record = false;
        if let Ok(mut ok) = RECORD_USER_PROBLEM_ARRAY.lock() {
            for record_num in 0..ok.len() {
                if ok[record_num].user_id == response_clone_for_record.submission.user_id
                    && ok[record_num].problem_id == response_clone_for_record.submission.problem_id
                {
                    ok[record_num]
                        .scores
                        .push(response_clone_for_record.score.clone());
                    if response_clone_for_record.result
                        == api_analysis::APIPostResponseResult::Accepted
                    {
                        let mut time = Vec::new();
                        let r = response_clone_for_record.cases.clone();
                        for num in 1..r.len() {
                            time.push(r[num].time);
                        }
                        ok[record_num].min_time = time;
                    }
                    find_record = true;
                    /*************************************************************************
                    【模块功能】               在得到数据后，进行存储处理，如果找到历史数据，则在其基础上更新
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    let mut highest_user_submit_add = true;
                    for passed_score in ok[record_num].scores.clone() {
                        if response_clone_for_record.score <= passed_score {
                            highest_user_submit_add = false;
                            break;
                        }
                    }
                    if highest_user_submit_add {
                        HIGHEST_USER_SUBMIT_ARRAY
                            .lock()
                            .unwrap()
                            .push(response_clone_for_record.submission.user_id.clone());
                    }
                    break;
                }
            }
            if !find_record {
                /*************************************************************************
                【模块功能】               在得到数据后，进行存储处理，如果未找到历史数据，则添加此新增数据
                【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                【更改记录】                2022-9-8 由林奕辰增加注释
                *************************************************************************/
                let mut new_record = api_analysis::RecordUserProblem::new();
                new_record.scores = Vec::new();
                new_record.scores.push(response_clone_for_record.score);
                HIGHEST_USER_SUBMIT_ARRAY
                    .lock()
                    .unwrap()
                    .push(response_clone_for_record.submission.user_id.clone());
                new_record.problem_id = response_clone_for_record.submission.problem_id;
                new_record.user_id = response_clone_for_record.submission.user_id;
                if response_clone_for_record.result == api_analysis::APIPostResponseResult::Accepted
                {
                    let mut time = Vec::new();
                    let r = response_clone_for_record.cases.clone();
                    for num in 1..r.len() {
                        time.push(r[num].time);
                    }
                    //println!("{:?}",time.clone());
                    new_record.min_time = time;
                }
                ok.push(new_record);
            }
            ALL_USER_SUBMIT_ARRAY
                .lock()
                .unwrap()
                .push(response_clone_for_record.submission.user_id.clone());
        }
        let the_contest_id = response_body.submission.contest_id;
        if the_contest_id > 0 {
            /*************************************************************************
            【模块功能】               如果是在比赛状态the_contest_id>0，则还需要根据比赛的需求
                                     存储对应比赛的内部数据，为便于后续比赛内的用户排行
            【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
            【更改记录】                2022-9-8 由林奕辰增加注释
            *************************************************************************/
            if let Ok(mut ok) = RECORD_CONTEST_USER_PROBLEM_ARRAY.lock() {
                for record_num in 0..ok.len() {
                    if ok[record_num].user_id == response_clone_for_record.submission.user_id
                        && ok[record_num].problem_id
                            == response_clone_for_record.submission.problem_id
                        && ok[record_num].contest_id == the_contest_id
                    {
                        ok[record_num]
                            .scores
                            .push(response_clone_for_record.score.clone());
                        if response_clone_for_record.result
                            == api_analysis::APIPostResponseResult::Accepted
                        {
                            let mut time = Vec::new();
                            let r = response_clone_for_record.cases.clone();
                            for num in 1..r.len() {
                                time.push(r[num].time);
                            }
                            ok[record_num].min_time = time;
                        }
                        find_record = true;
                        /*************************************************************************
                        【模块功能】               在得到数据后，进行存储处理，如果找到历史数据，则在其基础上更新
                        【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                        【更改记录】                2022-9-8 由林奕辰增加注释
                        *************************************************************************/
                        let mut highest_user_submit_add = true;
                        for passed_score in ok[record_num].scores.clone() {
                            if response_clone_for_record.score <= passed_score {
                                highest_user_submit_add = false;
                                break;
                            }
                        }
                        if highest_user_submit_add {
                            CONTEST_HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap().push((
                                the_contest_id,
                                response_clone_for_record.submission.user_id.clone(),
                            ));
                        }
                        break;
                    }
                }
                if !find_record {
                    /*************************************************************************
                    【模块功能】               在得到数据后，进行存储处理，如果未找到历史数据，则新增该数据点
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    let mut new_record = api_analysis::RecordContestUserProblem::new();
                    new_record.scores = Vec::new();
                    new_record.contest_id = response_clone_for_record.submission.contest_id;
                    new_record.scores.push(response_clone_for_record.score);
                    HIGHEST_USER_SUBMIT_ARRAY
                        .lock()
                        .unwrap()
                        .push(response_clone_for_record.submission.user_id.clone());
                    new_record.problem_id = response_clone_for_record.submission.problem_id;
                    new_record.user_id = response_clone_for_record.submission.user_id;
                    if response_clone_for_record.result
                        == api_analysis::APIPostResponseResult::Accepted
                    {
                        let mut time = Vec::new();
                        let r = response_clone_for_record.cases.clone();
                        for num in 1..r.len() {
                            time.push(r[num].time);
                        }
                        new_record.min_time = time;
                    }
                    ok.push(new_record);
                }
                CONTEST_ALL_USER_SUBMIT_ARRAY.lock().unwrap().push((
                    the_contest_id,
                    response_clone_for_record.submission.user_id.clone(),
                ));
            }
        }
    }
    if let Some(misc_data) = relevant_problem.clone().misc{
        if let Some(s)=misc_data.dynamic_ranking_ratio{
            response_body.score=response_body.score*(1.0-s);
        }
    }
    Ok(response_body)
}

//作为后台来处理post_jobs发送过来的数据，为了简化main.rs的部分函数而设置
/*************************************************************************
【函数名称】                waiting_dealer
【函数功能】                及时返回waiting信息
【参数】                   config:配置，api_body:提交的作答信息
【返回值】                 Result类型，如果异常则返回Error
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub async fn waiting_dealer(
    config_body: config_analysis::Config,
    api_body: api_analysis::APIPostJob,
) -> io::Result<api_analysis::APIPostResponse> {
    let mut relevant_problem = config_analysis::ConfigProblem::new();
    let mut problem_contain = false;
    for pro in config_body.problems {
        if pro.id == api_body.problem_id {
            relevant_problem = pro.clone();
            problem_contain = true;
            break;
        }
    }
    if !problem_contain {
        return Err(Error::new(ErrorKind::Other, "No Problem Found"));
    }
    let mut new_response = api_analysis::APIPostResponse::new();
    new_response.submission = api_body.clone();
    for num in 0..relevant_problem.cases.clone().len() + 1 {
        let mut case = api_analysis::APIPostResponseCase::new();
        case.id = num;
        case.result = api_analysis::APIPostResponseResult::Waiting;
        new_response.cases.push(case);
    }
    {
        let mut queue_arr = QUEUEING_RESPONSE_ARRAY.lock().unwrap();
        new_response.id = queue_arr.len();
    }
    running_dealer(new_response.clone());
    Ok(new_response)
}

//作为后台来处理post_jobs发送过来的数据，为了简化main.rs的部分函数而设置
/*************************************************************************
【函数名称】                running_dealer
【函数功能】                将数据放入任务运行队列，及时返回running信息
【参数】                   new_response：响应信息
【返回值】                 无
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
fn running_dealer(new_response: api_analysis::APIPostResponse) {
    let mut queue_arr = QUEUEING_RESPONSE_ARRAY.lock().unwrap();
    let mut new_running_response = new_response.clone();
    for num in 0..new_running_response.cases.len() {
        new_running_response.cases[num].result = api_analysis::APIPostResponseResult::Running;
    }
    new_running_response.state = api_analysis::APIPostResponseState::Running;
    new_running_response.result = api_analysis::APIPostResponseResult::Running;
    queue_arr.push_back(new_running_response.clone());
}

//作为后台来处理get_contest_id_rank_list发送过来的数据，为了简化main.rs的部分函数而设置
/*************************************************************************
【函数名称】                deal_get_contest_id_rank_list
【函数功能】                根据比赛的id，返回用户排名信息
【参数】                   info:目前场上用户作答部分信息，contest_id:比赛编号
【返回值】                 Responser类型，与get_contest_id_rank_list返回值同步
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub async fn deal_get_contest_id_rank_list(
    info: api_analysis::GetContestIDRankList,
    contest_id: usize,
) -> HttpResponse {
    if contest_id == 0 {
        let mut rank_list: Vec<api_analysis::UserRankResponse> = Vec::new();

        /*************************************************************************
        【模块功能】               判断算分模式是否为取最高分，并根据算分模式来获取用户分数
                                 包括highest和latest两种算分方法
        【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
        【更改记录】                2022-9-8 由林奕辰增加注释
        *************************************************************************/
        let mut highest = false;
        if let Some(s) = info.clone().scoring_rule {
            if s == api_analysis::RankListScoringRule::Highest {
                highest = true;
            }
        }
        let mut problem_array = PROBLEM_ID_ARRAY.lock().unwrap().clone();
        problem_array.sort();
        let user_problem_arr = RECORD_USER_PROBLEM_ARRAY.lock().unwrap().clone();
        let mut user_array = USER_ARRAY.lock().unwrap().clone();
        for user_num in 0..user_array.len() {
            let mut new_ranker = api_analysis::UserRankResponse::new();
            new_ranker.user = user_array[user_num].clone();
            for pro_num in problem_array.clone() {
                let mut push_zero = true;
                for mut up in user_problem_arr.clone() {
                    if up.user_id == user_array[user_num].id.unwrap() && up.problem_id == pro_num {
                        push_zero = false;
                        if highest {
                            up.scores.sort_by_key(|x| ((x.clone() * 100.0) as i32));
                        }
                        new_ranker.scores.push(up.scores.clone().pop().unwrap());
                        break;
                    }
                }
                if push_zero {
                    new_ranker.scores.push(0.0);
                }
            }
            rank_list.push(new_ranker);
        }

        match info.tie_breaker.clone() {
            /*************************************************************************
            【模块功能】               如果有打破平局的办法，则判断是哪种方法
            【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
            【更改记录】                2022-9-8 由林奕辰增加注释
            *************************************************************************/
            Some(s) => {
                if s == api_analysis::RankListTieBreaker::SubmissionTime {
                    /*************************************************************************
                    【模块功能】               用提交时间来判断，越早提交排名越高
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    let mut rank_map = BTreeMap::new();
                    rank_list.sort_by_key(|x| {
                        let user_problem_arr_clone = if highest {
                            HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap().clone()
                        } else {
                            ALL_USER_SUBMIT_ARRAY.lock().unwrap().clone()
                        };
                        let mut submit_rank = user_problem_arr_clone.len();
                        for up_num in 0..user_problem_arr_clone.len() {
                            if user_problem_arr_clone[up_num] == x.clone().user.id.unwrap() {
                                submit_rank = up_num;
                            }
                        }
                        rank_map.insert(x.clone().user.id.unwrap(), submit_rank);
                        submit_rank
                    });
                    // {
                    //     for i in rank_map.clone() {
                    //         println!("{}->{}", i.0, i.1);
                    //     }
                    // }
                    rank_list.reverse();
                    rank_list.sort_by_key(|x| {
                        let mut all = 0.0;
                        for i in x.clone().scores {
                            all += i;
                        }
                        ((all * 100.0) as i32)
                    });
                    rank_list.reverse();
                    for rank_num in 0..rank_list.len() {
                        rank_list[rank_num].rank = rank_num + 1;
                    }
                    for rank_num in 1..rank_list.len() {
                        let mut scores1 = 0.0;
                        let mut scores2 = 0.0;
                        for s1 in rank_list[rank_num - 1].scores.clone() {
                            scores1 += s1;
                        }
                        for s1 in rank_list[rank_num].scores.clone() {
                            scores2 += s1;
                        }
                        if scores1 == scores2
                            && rank_map[&rank_list[rank_num - 1].user.id.unwrap()]
                                == rank_map[&rank_list[rank_num].user.id.unwrap()]
                        {
                            rank_list[rank_num].rank = rank_list[rank_num - 1].rank;
                        }
                    }
                } else if s == api_analysis::RankListTieBreaker::SubmissionCount {
                    /*************************************************************************
                    【模块功能】               用提交次数来排名，越少成功提交排名越高
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    let mut rank_map = BTreeMap::new();
                    rank_list.sort_by_key(|x| {
                        let user_problem_arr_clone = ALL_USER_SUBMIT_ARRAY.lock().unwrap().clone();
                        let mut submit_rank = 0;
                        for up_num in 0..user_problem_arr_clone.len() {
                            if user_problem_arr_clone[up_num] == x.clone().user.id.unwrap() {
                                submit_rank += 1;
                            }
                        }
                        rank_map.insert(x.clone().user.id.unwrap(), submit_rank);
                        submit_rank
                    });
                    rank_list.reverse();
                    rank_list.sort_by_key(|x| {
                        let mut all = 0.0;
                        for i in x.clone().scores {
                            all += i;
                        }
                        ((all * 100.0) as i32)
                    });
                    rank_list.reverse();
                    for rank_num in 0..rank_list.len() {
                        rank_list[rank_num].rank = rank_num + 1;
                    }
                    for rank_num in 1..rank_list.len() {
                        let mut scores1 = 0.0;
                        let mut scores2 = 0.0;
                        for s1 in rank_list[rank_num - 1].scores.clone() {
                            scores1 += s1;
                        }
                        for s1 in rank_list[rank_num].scores.clone() {
                            scores2 += s1;
                        }
                        if scores1 == scores2
                            && rank_map[&rank_list[rank_num - 1].user.id.unwrap()]
                                == rank_map[&rank_list[rank_num].user.id.unwrap()]
                        {
                            rank_list[rank_num].rank = rank_list[rank_num - 1].rank;
                        }
                    }
                } else {
                    /*************************************************************************
                    【模块功能】               使用用户ID来排名，ID越小排名越高
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    rank_list.sort_by_key(|x| x.clone().user.id);
                    rank_list.reverse();
                    rank_list.sort_by_key(|x| {
                        let mut all = 0.0;
                        for i in x.clone().scores {
                            all += i;
                        }
                        ((all * 100.0) as i32)
                    });
                    rank_list.reverse();
                    for rank_num in 0..rank_list.len() {
                        rank_list[rank_num].rank = rank_num + 1;
                    }
                }
            }
            /*************************************************************************
            【模块功能】               如果没有打破平局的办法，则采用默认的排序方法，只用分数排序
            【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
            【更改记录】                2022-9-8 由林奕辰增加注释
            *************************************************************************/
            None => {
                // println!("None! just scores!");
                rank_list.sort_by_key(|x| x.clone().user.id);
                rank_list.reverse();
                rank_list.sort_by_key(|x| {
                    let mut all = 0.0;
                    for i in x.clone().scores {
                        all += i;
                    }
                    ((all * 100.0) as i32)
                });
                rank_list.reverse();
                for rank_num in 0..rank_list.len() {
                    rank_list[rank_num].rank = rank_num + 1;
                }
                for rank_num in 1..rank_list.len() {
                    let mut scores1 = 0.0;
                    let mut scores2 = 0.0;
                    for s1 in rank_list[rank_num - 1].scores.clone() {
                        scores1 += s1;
                    }
                    for s1 in rank_list[rank_num].scores.clone() {
                        scores2 += s1;
                    }
                    if scores1 == scores2 {
                        rank_list[rank_num].rank = rank_list[rank_num - 1].rank;
                    }
                }
            }
        }
        return HttpResponse::Ok().json(rank_list);
    } else {
        /*************************************************************************
        【模块功能】               比赛ID不为0，则开启比赛内部成员排序，不再使用全局成员排序
        【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
        【更改记录】                2022-9-8 由林奕辰增加注释
        *************************************************************************/
        let all_contests = CONTEST_ARRAY.lock().unwrap().clone();
        let mut find_contest = false;
        let mut this_contest = api_analysis::PostContest::new();
        for c in all_contests {
            if c.id.unwrap() == contest_id {
                find_contest = true;
                this_contest = c.clone();
                break;
            }
        }
        if !find_contest {
            let mut not_found_err = api_analysis::APIErr::new();
            not_found_err.reason = "ERR_NOT_FOUND".to_string();
            not_found_err.code = 3;
            not_found_err.message = "Contest 114514 not found.".to_string();
            return HttpResponse::NotFound().json(not_found_err);
        }

        let mut rank_list: Vec<api_analysis::UserRankResponse> = Vec::new();

        let mut highest = false;
        if let Some(s) = info.clone().scoring_rule {
            if s == api_analysis::RankListScoringRule::Highest {
                highest = true;
            }
        }
        let mut problem_array = this_contest.problem_ids.clone();
        let mut this_contest_id = this_contest.id.unwrap();
        let user_problem_arr_first = RECORD_CONTEST_USER_PROBLEM_ARRAY.lock().unwrap().clone();
        let mut user_problem_arr = Vec::new();
        for up in user_problem_arr_first {
            if up.contest_id == this_contest_id {
                user_problem_arr.push(up.clone());
            }
        }
        let mut user_array_first = USER_ARRAY.lock().unwrap().clone();
        let mut user_array = Vec::new();
        for u in user_array_first {
            if this_contest.user_ids.contains(&u.id.unwrap()) {
                user_array.push(u.clone());
            }
        }
        for user_num in 0..user_array.len() {
            let mut new_ranker = api_analysis::UserRankResponse::new();
            new_ranker.user = user_array[user_num].clone();
            for pro_num in problem_array.clone() {
                let mut push_zero = true;
                for mut up in user_problem_arr.clone() {
                    if up.user_id == user_array[user_num].id.unwrap() && up.problem_id == pro_num {
                        push_zero = false;
                        if highest {
                            up.scores.sort_by_key(|x| ((x.clone() * 100.0) as i32));
                        }
                        new_ranker.scores.push(up.scores.clone().pop().unwrap());
                        break;
                    }
                }
                if push_zero {
                    new_ranker.scores.push(0.0);
                }
            }
            rank_list.push(new_ranker);
        }

        match info.tie_breaker.clone() {
            /*************************************************************************
            【模块功能】               如果有打破平局的办法，则判断是哪种方法
            【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
            【更改记录】                2022-9-8 由林奕辰增加注释
            *************************************************************************/
            Some(s) => {
                if s == api_analysis::RankListTieBreaker::SubmissionTime {
                    /*************************************************************************
                    【模块功能】               用提交时间来排名，成功提交越早排名越高
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    let mut rank_map = BTreeMap::new();
                    rank_list.sort_by_key(|x| {
                        let user_problem_arr_clone = if highest {
                            let highest_arr_first =
                                CONTEST_HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap().clone();
                            let mut highest_arr = Vec::new();
                            for h in highest_arr_first {
                                if h.0 == this_contest_id {
                                    highest_arr.push(h.1);
                                }
                            }
                            highest_arr
                        } else {
                            let all_arr_first =
                                CONTEST_ALL_USER_SUBMIT_ARRAY.lock().unwrap().clone();
                            let mut all_arr = Vec::new();
                            for a in all_arr_first {
                                if a.0 == this_contest_id {
                                    all_arr.push(a.1);
                                }
                            }
                            all_arr
                        };
                        let mut submit_rank = user_problem_arr_clone.len();
                        for up_num in 0..user_problem_arr_clone.len() {
                            if user_problem_arr_clone[up_num] == x.clone().user.id.unwrap() {
                                submit_rank = up_num;
                            }
                        }
                        rank_map.insert(x.clone().user.id.unwrap(), submit_rank);
                        submit_rank
                    });
                    // {
                    //     for i in rank_map.clone() {
                    //         println!("{}->{}", i.0, i.1);
                    //     }
                    // }
                    rank_list.reverse();
                    rank_list.sort_by_key(|x| {
                        let mut all = 0.0;
                        for i in x.clone().scores {
                            all += i;
                        }
                        ((all * 100.0) as i32)
                    });
                    rank_list.reverse();
                    for rank_num in 0..rank_list.len() {
                        rank_list[rank_num].rank = rank_num + 1;
                    }
                    for rank_num in 1..rank_list.len() {
                        let mut scores1 = 0.0;
                        let mut scores2 = 0.0;
                        for s1 in rank_list[rank_num - 1].scores.clone() {
                            scores1 += s1;
                        }
                        for s1 in rank_list[rank_num].scores.clone() {
                            scores2 += s1;
                        }
                        if scores1 == scores2
                            && rank_map[&rank_list[rank_num - 1].user.id.unwrap()]
                                == rank_map[&rank_list[rank_num].user.id.unwrap()]
                        {
                            rank_list[rank_num].rank = rank_list[rank_num - 1].rank;
                        }
                    }
                } else if s == api_analysis::RankListTieBreaker::SubmissionCount {
                    /*************************************************************************
                    【模块功能】               使用提交次数排名，成功提交次数越少排名越靠前
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    let mut rank_map = BTreeMap::new();
                    rank_list.sort_by_key(|x| {
                        let user_problem_arr_clone = {
                            let all_arr_first =
                                CONTEST_ALL_USER_SUBMIT_ARRAY.lock().unwrap().clone();
                            let mut all_arr = Vec::new();
                            for a in all_arr_first {
                                if a.0 == this_contest_id {
                                    all_arr.push(a.1);
                                }
                            }
                            all_arr
                        };
                        let mut submit_rank = 0;
                        for up_num in 0..user_problem_arr_clone.len() {
                            if user_problem_arr_clone[up_num] == x.clone().user.id.unwrap() {
                                submit_rank += 1;
                            }
                        }
                        rank_map.insert(x.clone().user.id.unwrap(), submit_rank);
                        submit_rank
                    });
                    rank_list.reverse();
                    rank_list.sort_by_key(|x| {
                        let mut all = 0.0;
                        for i in x.clone().scores {
                            all += i;
                        }
                        ((all * 100.0) as i32)
                    });
                    rank_list.reverse();
                    for rank_num in 0..rank_list.len() {
                        rank_list[rank_num].rank = rank_num + 1;
                    }
                    for rank_num in 1..rank_list.len() {
                        let mut scores1 = 0.0;
                        let mut scores2 = 0.0;
                        for s1 in rank_list[rank_num - 1].scores.clone() {
                            scores1 += s1;
                        }
                        for s1 in rank_list[rank_num].scores.clone() {
                            scores2 += s1;
                        }
                        if scores1 == scores2
                            && rank_map[&rank_list[rank_num - 1].user.id.unwrap()]
                                == rank_map[&rank_list[rank_num].user.id.unwrap()]
                        {
                            rank_list[rank_num].rank = rank_list[rank_num - 1].rank;
                        }
                    }
                } else {
                    /*************************************************************************
                    【模块功能】               使用用户ID来排名，ID越小排名越高
                    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                    【更改记录】                2022-9-8 由林奕辰增加注释
                    *************************************************************************/
                    rank_list.sort_by_key(|x| x.clone().user.id);
                    rank_list.reverse();
                    rank_list.sort_by_key(|x| {
                        let mut all = 0.0;
                        for i in x.clone().scores {
                            all += i;
                        }
                        ((all * 100.0) as i32)
                    });
                    rank_list.reverse();
                    for rank_num in 0..rank_list.len() {
                        rank_list[rank_num].rank = rank_num + 1;
                    }
                }
            }
            None => {
                /*************************************************************************
                【模块功能】               如果没有打破平局的办法，则采用默认的排序方法，只用分数排序
                【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
                【更改记录】                2022-9-8 由林奕辰增加注释
                *************************************************************************/
                // println!("None! just scores!");
                rank_list.sort_by_key(|x| x.clone().user.id);
                rank_list.reverse();
                rank_list.sort_by_key(|x| {
                    let mut all = 0.0;
                    for i in x.clone().scores {
                        all += i;
                    }
                    ((all * 100.0) as i32)
                });
                rank_list.reverse();
                for rank_num in 0..rank_list.len() {
                    rank_list[rank_num].rank = rank_num + 1;
                }
                for rank_num in 1..rank_list.len() {
                    let mut scores1 = 0.0;
                    let mut scores2 = 0.0;
                    for s1 in rank_list[rank_num - 1].scores.clone() {
                        scores1 += s1;
                    }
                    for s1 in rank_list[rank_num].scores.clone() {
                        scores2 += s1;
                    }
                    if scores1 == scores2 {
                        rank_list[rank_num].rank = rank_list[rank_num - 1].rank;
                    }
                }
            }
        }
        return HttpResponse::Ok().json(rank_list);
    }
}

//竞争得分作为一种特殊的获取rank_list的情况，特殊判断，为减少main.rs的行数而设置
/*************************************************************************
【函数名称】                ranking_ratio
【函数功能】                在特殊情况（竞争得分）下，计算用户排名的函数
【参数】                   rank_vec：目前的配置下的相关题目
【返回值】                 Result类型，返回用户排名列表
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub async fn ranking_ratio(
    rank_vec: Vec<(config_analysis::ConfigProblem, f32)>,
) -> std::io::Result<Vec<api_analysis::UserRankResponse>> {
    let mut user_name = USER_ARRAY.lock().unwrap().clone();
    let mut user_rank = Vec::new();
    let mut user_problem = RECORD_USER_PROBLEM_ARRAY.lock().unwrap().clone();
    let mut vec_scores = Vec::new();
    // println!("{:?}",user_problem.clone());
    // println!("rank_vec:{:?}",rank_vec.clone());
    /*************************************************************************
    【模块功能】               对于每个问题的每个测试点，选取最短时，并且将用户的分数按照
                            与最短时与其比来计算，将分数放入矩阵中：Vec<Vec<usize>>
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    for pro_num in 0..rank_vec.len() {
        let mut tmp_vec = Vec::new();
        for up in user_problem.clone() {
            if up.problem_id == rank_vec[pro_num].0.id && up.min_time.len() != 0 {
                tmp_vec.push(up.clone());
            }
        }
        // println!("tmp_vec:{:?}",tmp_vec.clone());
        let mut min_vec = tmp_vec[0].min_time.clone();
        for num1 in 0..tmp_vec.len() {
            for num2 in 0..min_vec.len() {
                if tmp_vec[num1].min_time[num2] < min_vec[num2] {
                    min_vec[num2] = tmp_vec[num1].min_time[num2];
                }
            }
        }
        // println!("min_vec:{:?}",min_vec.clone());
        let mut tmp_score = Vec::new();
        // println!("users:{:?}",user_name.clone());
        for num1 in 0..user_name.len() {
            let mut find_user_pro = false;
            for num2 in 0..tmp_vec.len() {
                if tmp_vec[num2].problem_id == rank_vec[pro_num].0.id
                    && tmp_vec[num2].user_id == user_name[num1].id.unwrap()
                {
                    find_user_pro = true;
                    let mut score = 100.0 * (1.0 - rank_vec[pro_num].1);
                    for num3 in 0..tmp_vec[num2].min_time.len() {
                        score += 100.0 * rank_vec[pro_num].1 / (min_vec.len() as f32)
                            * (min_vec[num3] as f32 / tmp_vec[num2].min_time[num3] as f32)
                    }
                    tmp_score.push(score);
                    // println!("{}",score);
                    break;
                }
            }
            if !find_user_pro {
                let mut find_again = false;
                for num4 in user_problem.clone() {
                    if num4.user_id == user_name[num1].id.unwrap()
                        && num4.problem_id == rank_vec[pro_num].0.id
                    {
                        let mut s = 0.0;
                        for i in num4.scores.clone() {
                            s += i;
                        }
                        tmp_score.push(s * (1.0 - rank_vec[pro_num].1));
                        find_again = true;
                    }
                }
                if !find_again {
                    tmp_score.push(0.0);
                }
            }
        }
        vec_scores.push(tmp_score);
    }
    /*************************************************************************
    【模块功能】               从最短时矩阵中提取用户分数，与用户身份匹配，进行排行榜返回
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    for num in 0..user_name.len() {
        let mut v = Vec::new();
        for x in vec_scores.clone() {
            v.push(x[num]);
        }
        let mut ur = api_analysis::UserRankResponse::new();
        ur.user = user_name[num].clone();
        ur.scores = v.clone();
        user_rank.push(ur);
    }

    /*************************************************************************
    【模块功能】               使用分数来排名，相同分数下则用用户ID来排名（一般分数不同）
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    user_rank.sort_by_key(|x| x.user.id.unwrap());
    user_rank.reverse();
    user_rank.sort_by_key(|x| {
        let mut score_all = 0.0;
        for i in x.scores.clone() {
            score_all += i;
        }
        (score_all * 1000.0) as i32
    });
    user_rank.reverse();
    for num in 0..user_rank.len() {
        user_rank[num].rank = num + 1;
    }
    Ok(user_rank)
}
