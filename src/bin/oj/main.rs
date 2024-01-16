use oj::api_analysis;
use oj::config_analysis;
use oj::trail_terminal;
use oj::{
    ALL_USER_SUBMIT_ARRAY, API_ARRAY, CONTEST_ALL_USER_SUBMIT_ARRAY, CONTEST_ARRAY,
    CONTEST_HIGHEST_USER_SUBMIT_ARRAY, HIGHEST_USER_SUBMIT_ARRAY, PROBLEM_ID_ARRAY,
    QUEUEING_RESPONSE_ARRAY, RECORD_CONTEST_USER_PROBLEM_ARRAY, RECORD_USER_PROBLEM_ARRAY,
    RESPONSE_ARRAY, USER_ARRAY, USER_ID_ARRAY, USER_NAME_ARRAY,
};
use oj::file_analysis::{file_flush, file_read, file_save};
use actix_web::{
    get, middleware::Logger, post, put, web, App, HttpResponse, HttpServer, Responder,
};
use chrono;
use env_logger;
use log;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read};
use std::process::{Command, Stdio};
use std::string::String;
use std::{env, io};
use std::hash::Hasher;
use wait_timeout::ChildExt;
use crate::api_analysis::UserRankResponse;
use chrono::NaiveDateTime;
use std::sync::{Arc, Mutex};
#[macro_use]
extern crate lazy_static;


/*************************************************************************
【函数名称】                greet
【函数功能】                测试网络功能get是否正常
【参数】                   name：用户姓名
【返回值】                 Responser类型，返回响应结果
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    log::info!(target: "greet_handler", "Greeting {}", name);
    format!("Hello {name}!")
}

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}

/*************************************************************************
【函数名称】                deal_post_jobs
【函数功能】                作为辅助post_jobs进行数据处理的异步多线程内函数
【参数】                   config：配置，api：发送的job内容，格式为json
【返回值】                 无
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
fn deal_post_jobs(config_body: config_analysis::Config, api_body: api_analysis::APIPostJob) {
    // println!("new async function to deal with the post");
    match trail_terminal::api_terminal(config_body.clone(), api_body.clone()) {
        Ok(mut o) => {
            match RESPONSE_ARRAY.lock() {
                Ok(mut ok) => {
                    // println!("ok::::{}", o.clone().id);
                    o.id = ok.len();
                    let mut new_api_post_response = o.clone();
                    ok.push_back(new_api_post_response);
                    //println!("{:?}",ok.clone());
                }
                Err(_err) => {
                    return;
                    //HttpResponse::Conflict().finish();
                }
            }
            if let Err(e) = file_save() {
                return;
                //HttpResponse::Conflict().finish();
            }
            // HttpResponse::Ok().json(o)
        }
        Err(e) => {
            println!("Error from the trail_terminal");
            log::info!("{}", e.to_string());
            let mut ok = RESPONSE_ARRAY.lock().unwrap();
            let mut err = api_analysis::APIPostResponse::new();
            err.result = api_analysis::APIPostResponseResult::SPJError;
            err.state = api_analysis::APIPostResponseState::Finished;
            err.submission = api_body.clone();
            ok.push_back(err);
            //HttpResponse::NotFound().finish()
        }
    }
}

/*************************************************************************
【函数名称】                post_jobs
【函数功能】                获取post发送的用户作答job,并返回相应的响应
【参数】                   config：配置，api：发送的job内容，格式为json
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则立即返回Queueing
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[post("/jobs")]
async fn post_jobs(
    api_body: web::Json<api_analysis::APIPostJob>,
    config_body: web::Data<config_analysis::Config>,
) -> impl Responder {
    log::info!("From User ID:{}", api_body.user_id);
    let mut api_body_simple_check = api_body.0.clone();
    let api_body = api_body_simple_check.clone();
    let config_body = config_body.get_ref().clone();
    match USER_ID_ARRAY.lock() {
        Ok(ok) => {
            let mut problem_id_arr = PROBLEM_ID_ARRAY.lock().unwrap().clone();
            if !problem_id_arr.contains(&api_body_simple_check.problem_id) {
                let mut not_found = api_analysis::APIErr::new();
                not_found.code = 3;
                not_found.reason = "ERR_NOT_FOUND".to_string();
                return HttpResponse::NotFound().json(not_found);
            }
            if !ok.contains(&api_body_simple_check.user_id) {
                let mut not_found = api_analysis::APIErr::new();
                not_found.code = 3;
                not_found.reason = "ERR_NOT_FOUND".to_string();
                return HttpResponse::NotFound().json(not_found);
            }
            if api_body_simple_check.contest_id > 0 {
                let mut contest_check = false;
                let contest_arr = CONTEST_ARRAY.lock().unwrap().clone();
                for contest in contest_arr {
                    if contest.id.unwrap() == api_body_simple_check.contest_id {
                        let contest_check_more = contest.clone();
                        if contest_check_more
                            .user_ids
                            .contains(&api_body_simple_check.user_id)
                            && contest_check_more
                                .problem_ids
                                .contains(&api_body_simple_check.problem_id)
                        {
                            //在这里再检查一遍，看看是否超过了比赛要求的时间和次数限制
                            if let Ok(ok) = chrono::NaiveDateTime::parse_from_str(
                                contest_check_more.from.as_str(),
                                "%Y-%m-%dT%H:%M:%S%.3fZ",
                            ) {
                                if chrono::NaiveDateTime::parse_from_str(
                                    chrono::prelude::Utc::now()
                                        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                                        .to_string()
                                        .as_str(),
                                    "%Y-%m-%dT%H:%M:%S%.3fZ",
                                )
                                .unwrap()
                                    < ok
                                {
                                    let mut not_found = api_analysis::APIErr::new();
                                    not_found.code = 1;
                                    not_found.reason = "ERR_INVALID_ARGUMENT".to_string();
                                    return HttpResponse::BadRequest().json(not_found);
                                }
                            } else {
                                return HttpResponse::Conflict().finish();
                            }
                            if let Ok(ok) = chrono::NaiveDateTime::parse_from_str(
                                contest_check_more.to.as_str(),
                                "%Y-%m-%dT%H:%M:%S%.3fZ",
                            ) {
                                if chrono::NaiveDateTime::parse_from_str(
                                    chrono::prelude::Utc::now()
                                        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                                        .to_string()
                                        .as_str(),
                                    "%Y-%m-%dT%H:%M:%S%.3fZ",
                                )
                                .unwrap()
                                    > ok
                                {
                                    let mut not_found = api_analysis::APIErr::new();
                                    not_found.code = 1;
                                    not_found.reason = "ERR_INVALID_ARGUMENT".to_string();
                                    return HttpResponse::BadRequest().json(not_found);
                                }
                            } else {
                                return HttpResponse::Conflict().finish();
                            }
                            let mut poss_time = 0;
                            let api_poss_arr = API_ARRAY.lock().unwrap().clone();
                            for single_poss in api_poss_arr {
                                if single_poss.contest_id == api_body_simple_check.contest_id
                                    && single_poss.user_id == api_body_simple_check.user_id
                                {
                                    poss_time += 1;
                                }
                            }
                            // println!("poss time: {}", poss_time);
                            if poss_time < contest_check_more.submission_limit {
                                contest_check = true;
                            } else {
                                let mut not_found = api_analysis::APIErr::new();
                                not_found.code = 4;
                                not_found.reason = "ERR_RATE_LIMIT".to_string();
                                return HttpResponse::BadRequest().json(not_found);
                            }
                        }
                        break;
                    }
                }
                if !contest_check {
                    let mut not_found = api_analysis::APIErr::new();
                    not_found.code = 1;
                    not_found.reason = "ERR_INVALID_ARGUMENT".to_string();
                    return HttpResponse::BadRequest().json(not_found);
                }
            }
        }
        Err(_err) => {
            return HttpResponse::Conflict().finish();
        }
    }
    {
        match API_ARRAY.lock() {
            Ok(mut o) => {
                o.push_back(api_body.clone());
            }
            Err(_e) => {
                return HttpResponse::Conflict().finish();
            }
        }
    }
    let config_body_copy = config_body.clone();
    let api_body_copy = api_body.clone();
    web::block(move || deal_post_jobs(config_body.clone(), api_body.clone()));
    match trail_terminal::waiting_dealer(config_body_copy.clone(), api_body_copy.clone()).await {
        Ok(o) => return HttpResponse::Ok().json(o),
        Err(e) => return HttpResponse::Conflict().finish(),
    }
}

/*************************************************************************
【函数名称】                get_jobs
【函数功能】                获取请求get,并返回相应的全部任务列表
【参数】                   info：get的相关参数，如时间等
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则立即返回获得的任务列表，json形式
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/jobs")]
async fn get_jobs(info: web::Query<api_analysis::GetJobInfo>) -> impl Responder {
    if let Err(e) = file_read() {
        return HttpResponse::Conflict().finish();
    }
    log::info!("Get Jobs' Info!");
    // println!("get jobs info!");
    // println!("{:?}", info.0.clone());
    let mut json_arrays = Vec::new();
    //decide if the time is valid
    if let Some(s) = &info.from {
        if let Err(e) = chrono::NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%dT%H:%M:%S%.3fZ")
        {
            return HttpResponse::BadRequest().finish();
        }
    }
    if let Some(s) = &info.to {
        if let Err(e) = chrono::NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%dT%H:%M:%S%.3fZ")
        {
            return HttpResponse::BadRequest().finish();
        }
    }
    //end judge time
    match RESPONSE_ARRAY.lock() {
        Ok(mut ok) => {
            // println!("{:?}", ok.clone());
            for api_post_response in ok.clone() {
                if api_analysis::get_job_judge(&info.0, &api_post_response.clone()) {
                    json_arrays.push(api_post_response.clone());
                }
            }
        }
        Err(_err) => {
            return HttpResponse::Conflict().finish();
        }
    }
    HttpResponse::Ok().json(json_arrays)
}

/*************************************************************************
【函数名称】                get_jobs_id
【函数功能】                根据特定的任务编号返回任务信息
【参数】                   job_id:任务编号
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则返回相应的任务，json形式
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/jobs/{job_id}")]
async fn get_jobs_id(job_id: web::Path<usize>) -> impl Responder {
    let job_id = job_id.into_inner();
    match RESPONSE_ARRAY.lock() {
        Ok(ok) => {
            if ok.len() < job_id + 1 {
                let queueing = QUEUEING_RESPONSE_ARRAY.lock().unwrap().clone();
                //println!("{:?}",queueing.clone());
                for q in queueing {
                    if q.id == job_id {
                        return HttpResponse::Ok().json(q.clone());
                    }
                }
                return HttpResponse::NotFound().reason("ERR_NOT_FOUND").finish();
            } else {
                return HttpResponse::Ok().json(ok[job_id].clone());
            }
        }
        Err(_err) => {
            return HttpResponse::Conflict().finish();
        }
    }
}

/*************************************************************************
【函数名称】                put_jobs_id
【函数功能】                重新测评某任务，一般用于config参数改变后
【参数】                   config：配置，job_id:重新测评的任务的id编号
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则返回重新测评后的新结果
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[put("/jobs/{job_id}")]
async fn put_jobs_id(
    job_id: web::Path<usize>,
    config_body: web::Data<config_analysis::Config>,
) -> impl Responder {
    let job_id = job_id.into_inner();
    match RESPONSE_ARRAY.lock() {
        Ok(mut ok) => {
            if ok.len() < job_id + 1 {
                let mut not_found_err = api_analysis::APIErr::new();
                not_found_err.reason = "ERR_NOT_FOUND".to_string();
                not_found_err.code = 3;
                not_found_err.message = "Job ".to_string() + &job_id.to_string() + " not found.";
                return HttpResponse::NotFound().json(not_found_err);
            } else {
                let api_post_response_clone = ok[job_id].clone();
                let api_post_job_update = api_post_response_clone.submission;
                let created_time_save = api_post_response_clone.created_time;
                let id_save = api_post_response_clone.id;
                match trail_terminal::api_terminal(
                    config_body.get_ref().clone(),
                    api_post_job_update,
                ) {
                    Ok(mut o) => {
                        o.created_time = created_time_save;
                        o.id = id_save;
                        ok[job_id] = o.clone();
                        HttpResponse::Ok().json(o)
                    }
                    Err(e) => {
                        log::info!("{}", e.to_string());
                        HttpResponse::NotFound().finish()
                    }
                }
            }
        }
        Err(_err) => {
            println!("lock wrong");
            return HttpResponse::Conflict().finish();
        }
    }
}

/*************************************************************************
【函数名称】                put_users
【函数功能】                获取post发送的用户数据，进行用户名变更或者新建用户
【参数】                   user_body:相应的用户参数
【返回值】                 Responser类型，若用户已存在则返回BadRequest等，
                         否则则返回创建成功后用户的新编号、姓名
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[post("/users")]
async fn post_users(user_body: web::Json<api_analysis::APIPostUsers>) -> impl Responder {
    let mut user_body_clone = user_body.0.clone();
    {
        match USER_ID_ARRAY.lock() {
            Ok(o) => {
                let mut id_array = o.clone();
                if let Some(s) = user_body_clone.id {
                    if !id_array.contains(&s) {
                        let mut not_found = api_analysis::APIErr::new();
                        not_found.code = 3;
                        not_found.reason = "ERR_NOT_FOUND".to_string();
                        not_found.message =
                            "User ".to_string() + &s.to_string().to_string() + " not found";
                        return HttpResponse::NotFound().json(not_found);
                    }
                }
            }
            Err(_err) => {
                println!("lock wrong");
                return HttpResponse::Conflict().finish();
            }
        }
    }
    match USER_NAME_ARRAY.lock() {
        Ok(mut ok) => {
            if ok.contains(&user_body_clone.name) {
                let mut bad_request = api_analysis::APIErr::new();
                bad_request.code = 1;
                bad_request.reason = "ERR_INVALID_ARGUMENT".to_string();
                bad_request.message = "User name ".to_string()
                    + user_body_clone.name.clone().as_str()
                    + " already exists.";
                return HttpResponse::BadRequest().json(bad_request);
            } else {
                match user_body_clone.id {
                    Some(some_id) => match USER_ID_ARRAY.lock() {
                        Ok(ok1) => {
                            if ok1.contains(&some_id) {
                                match USER_ARRAY.lock() {
                                    Ok(mut ok2) => {
                                        let mut user_array = ok2.clone();
                                        let mut delete_num = 0;
                                        for i in 0..user_array.len() {
                                            if some_id == user_array[i].id.unwrap() {
                                                delete_num = i;
                                                ok2[i].name = user_body_clone.name.clone();
                                            }
                                        }
                                        ok.remove(delete_num);
                                        ok.push(user_body_clone.name.clone());
                                        return HttpResponse::Ok().json(user_body_clone);
                                    }
                                    Err(_err) => {
                                        println!("lock wrong");
                                        return HttpResponse::Conflict().finish();
                                    }
                                }
                            } else {
                                let mut not_found = api_analysis::APIErr::new();
                                not_found.code = 3;
                                not_found.reason = "ERR_NOT_FOUND".to_string();
                                not_found.message =
                                    "User ".to_string() + &some_id.to_string() + " not found";
                                return HttpResponse::NotFound().json(not_found);
                            }
                        }
                        Err(_err) => {
                            println!("lock wrong");
                            return HttpResponse::Conflict().finish();
                        }
                    },
                    None => match USER_ID_ARRAY.lock() {
                        Ok(mut ok3) => {
                            let mut id_array: Vec<_> = ok3.clone();
                            id_array.sort();
                            let mut max_id = 0;
                            if let Some(s) = id_array.pop() {
                                max_id = s.clone() + 1;
                            }
                            ok3.push(max_id);
                            ok.push(user_body_clone.name.clone());
                            match USER_ARRAY.lock() {
                                Ok(mut ok4) => ok4.push(api_analysis::APIPostUsers {
                                    id: Some(max_id),
                                    name: user_body_clone.name.clone(),
                                }),
                                Err(_err) => {
                                    println!("lock wrong");
                                    return HttpResponse::Conflict().finish();
                                }
                            }
                            user_body_clone.id = Some(max_id);
                            return HttpResponse::Ok().json(user_body_clone);
                        }
                        Err(_err) => {
                            println!("lock wrong");
                            return HttpResponse::Conflict().finish();
                        }
                    },
                }
            }
        }
        Err(_err) => {
            println!("lock wrong");
            return HttpResponse::Conflict().finish();
        }
    }
}

/*************************************************************************
【函数名称】                get_users
【函数功能】                获取目前全部的用户列表
【参数】                   无
【返回值】                 Responser类型，若异常则返回Conflict等，否则返回用户列表
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/users")]
async fn get_users() -> impl Responder {
    match USER_ARRAY.lock() {
        Ok(ok) => {
            let return_users_array = ok.clone();
            return HttpResponse::Ok().json(return_users_array);
        }
        Err(_err) => {
            // println!("lock wrong");
            return HttpResponse::Conflict().finish();
        }
    }
}

//比赛模式-----------------------------------------------

/*************************************************************************
【函数名称】                post_contests
【函数功能】                获取post发送的新的比赛配置,并返回相应的比赛参数
【参数】                   contest_body:为比赛参数，json形式
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则返回比赛的相关参数，比赛编号不应为0
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[post("/contests")]
async fn post_contests(contest_body: web::Json<api_analysis::PostContest>) -> impl Responder {
    match CONTEST_ARRAY.lock() {
        Ok(mut ok) => {
            match contest_body.id {
                Some(some_id) => {
                    let mut contests_array = ok.clone();
                    for contest_num in 0..ok.len() {
                        if contests_array[contest_num].id.unwrap() == some_id {
                            let id_contains = USER_ID_ARRAY.lock().unwrap().clone();
                            let pro_contains = PROBLEM_ID_ARRAY.lock().unwrap().clone();
                            for single_id in contest_body.clone().user_ids {
                                if !id_contains.contains(&single_id) {
                                    let mut not_found_err = api_analysis::APIErr::new();
                                    not_found_err.reason = "ERR_NOT_FOUND".to_string();
                                    not_found_err.code = 3;
                                    not_found_err.message = "Contest ".to_string()
                                        + &some_id.to_string()
                                        + " not found.";
                                    return HttpResponse::NotFound().json(not_found_err);
                                }
                            }
                            for single_id in contest_body.clone().problem_ids {
                                if !pro_contains.contains(&single_id) {
                                    let mut not_found_err = api_analysis::APIErr::new();
                                    not_found_err.reason = "ERR_NOT_FOUND".to_string();
                                    not_found_err.code = 3;
                                    not_found_err.message = "Contest ".to_string()
                                        + &some_id.to_string()
                                        + " not found.";
                                    return HttpResponse::NotFound().json(not_found_err);
                                }
                            }
                            ok[contest_num] = contest_body.0.clone();
                            return HttpResponse::Ok().json(contest_body.0);
                        }
                    }
                    //如果之前没有成功返回Ok.json()的话↓
                    let mut not_found_err = api_analysis::APIErr::new();
                    not_found_err.reason = "ERR_NOT_FOUND".to_string();
                    not_found_err.code = 3;
                    not_found_err.message =
                        "Contest ".to_string() + &some_id.to_string() + " not found.";
                    return HttpResponse::NotFound().json(not_found_err);
                }
                None => {
                    let id_contains = USER_ID_ARRAY.lock().unwrap().clone();
                    let pro_contains = PROBLEM_ID_ARRAY.lock().unwrap().clone();
                    for single_id in contest_body.clone().user_ids {
                        if !id_contains.contains(&single_id) {
                            let mut not_found_err = api_analysis::APIErr::new();
                            not_found_err.reason = "ERR_NOT_FOUND".to_string();
                            not_found_err.code = 3;
                            not_found_err.message = "Contest 114514 not found.".to_string();
                            return HttpResponse::NotFound().json(not_found_err);
                        }
                    }
                    for single_id in contest_body.clone().problem_ids {
                        if !pro_contains.contains(&single_id) {
                            let mut not_found_err = api_analysis::APIErr::new();
                            not_found_err.reason = "ERR_NOT_FOUND".to_string();
                            not_found_err.code = 3;
                            not_found_err.message = "Contest 114514 not found.".to_string();
                            return HttpResponse::NotFound().json(not_found_err);
                        }
                    }
                    let mut contest_body_copy = contest_body.0.clone();
                    contest_body_copy.id = Some(ok.len() + 1);
                    ok.push(contest_body_copy.clone());
                    return HttpResponse::Ok().json(contest_body_copy.clone());
                }
            }
        }
        Err(_err) => {
            // println!("lock wrong");
            return HttpResponse::Conflict().finish();
        }
    }
}

/*************************************************************************
【函数名称】                get_contests
【函数功能】                获取目前已有的全部比赛列表
【参数】                   无
【返回值】                 Responser类型，返回目前的比赛列表，json形式
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/contests")]
async fn get_contests() -> impl Responder {
    let contests_array = CONTEST_ARRAY.lock().unwrap().clone();
    HttpResponse::Ok().json(contests_array)
}

/*************************************************************************
【函数名称】                get_contests_id
【函数功能】                根据id来返回对应的比赛信息
【参数】                   contest_id:比赛的id
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则返回比赛的相关参数，比赛编号不应为0
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/contests/{contest_id}")]
async fn get_contest_id(contest_id: web::Path<usize>) -> impl Responder {
    let contests_array = CONTEST_ARRAY.lock().unwrap().clone();
    let contest_id = contest_id.into_inner();
    for contest_num in 0..contests_array.len() {
        if contests_array[contest_num].id.unwrap() == contest_id {
            return HttpResponse::Ok().json(contests_array[contest_num].clone());
        }
    }
    let mut not_found_err = api_analysis::APIErr::new();
    not_found_err.reason = "ERR_NOT_FOUND".to_string();
    not_found_err.code = 3;
    not_found_err.message = "Contest 114514 not found.".to_string();
    return HttpResponse::NotFound().json(not_found_err);
}


/*************************************************************************
【函数名称】                get_contest_id_rank_list
【函数功能】                根据比赛id来返回现有的比赛排行榜
【参数】                   contest_id:为比赛id
【返回值】                 Responser类型，若异常则返回Conflict、NotFound等，
                         否则返回比赛的相关参数，比赛编号为0时返回全部的用户排行，
                         比赛id不为0时返回相应的比赛内用户排行，不同情况算法较为复杂
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[get("/contests/{contest_id}/ranklist")]
async fn get_contest_id_rank_list(
    config_body: web::Data<config_analysis::Config>,
    contest_id: web::Path<usize>,
    info: web::Query<api_analysis::GetContestIDRankList>,
) -> impl Responder {
    let info = info.0.clone();
    let contest_id = contest_id.into_inner();
    //竞争得分开始
    {
        let mut rank_ratio_vec = Vec::new();
        let mut if_rank_ratio = false;
        for c in config_body.get_ref().clone().problems {
            if let Some(s) = c.clone().misc {
                if let Some(ratio) = s.dynamic_ranking_ratio {
                    if_rank_ratio = true;
                    rank_ratio_vec.push((c.clone(), ratio));
                }
            }
        }
        if if_rank_ratio {
            if let Ok(o) = trail_terminal::ranking_ratio(rank_ratio_vec).await {
                return HttpResponse::Ok().json(o);
            } else {
                return HttpResponse::Conflict().finish();
            }
        }
    }
    //竞争得分结束

    //该函数代码较长，原因在于情况较为复杂，且上手较早，拆分代码至函数的话也很难复用，并且各部分代码的紧密度较高，因此放置一起
    //该函数为特殊情况，因为它要确保支持比赛，而各种比赛的数据很复杂
    //为进行main.rs的结构简化，将deal_get_contest_id_rank_list移动到trail_terminal中进行处理
    trail_terminal::deal_get_contest_id_rank_list(info,contest_id).await
}
//比赛模式结束-----------------------------------------------



/*************************************************************************
【函数名称】                main
【函数功能】                获取命令行参数-c指定配置文件，-f清除遗留数据，APP打开网络连接，
                         并使所有main.rs中的函数运行
【参数】                   无
【返回值】                 如果没有-c或者-c后无文件/文件不合法，则会报错退出
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args: Vec<_> = env::args().collect();
    let mut find_config = false;
    let mut oj_config = config_analysis::Config::new();
    for arg_num in 0..args.len() {
        if args[arg_num] == "-c" || args[arg_num] == "--config" {
            if arg_num == args.len() - 1 {
                return Err(Error::new(ErrorKind::InvalidInput, "No Config File!"));
            } else {
                match File::open(args[arg_num + 1].clone()) {
                    Err(e) => {
                        return Err(e);
                    }
                    Ok(o) => {
                        oj_config = serde_json::from_reader(BufReader::new(o))?;
                        find_config = true;
                        match PROBLEM_ID_ARRAY.lock() {
                            Ok(mut ok) => {
                                let oj_config_clone = oj_config.clone();
                                for i in oj_config_clone.problems {
                                    ok.push(i.id);
                                }
                            }
                            Err(_err) => {
                                return Err(Error::new(
                                    ErrorKind::UnexpectedEof,
                                    "problem id lock wrong in main",
                                ));
                            }
                        }
                    }
                }
            }
        } else if args[arg_num] == "-f" || args[arg_num] == "--flush-data" {
            file_flush()?;
            // println!("I have cleared the data of OJ!");
        }
    }
    if !find_config {
        return Err(Error::new(ErrorKind::InvalidInput, "No -c or --config!"));
    }

    match USER_ARRAY.lock() {
        Ok(mut ok) => {
            ok.push(api_analysis::APIPostUsers {
                id: Some(0),
                name: String::from("root"),
            });
        }
        Err(_err) => {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "user lock wrong in main",
            ));
        }
    }
    match USER_NAME_ARRAY.lock() {
        Ok(mut ok) => {
            ok.push(String::from("root"));
        }
        Err(_err) => {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "name lock wrong in main",
            ));
        }
    }
    match USER_ID_ARRAY.lock() {
        Ok(mut ok) => {
            ok.push(0);
        }
        Err(_err) => {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "id lock wrong in main",
            ));
        }
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .app_data(web::Data::new(oj_config.clone()))
            .service(greet)
            // DO NOT REMOVE: used in automatic testing
            .service(exit)
            .service(post_jobs)
            .service(get_jobs)
            .service(get_jobs_id)
            .service(put_jobs_id)
            .service(post_users)
            .service(get_users)
            .service(post_contests)
            .service(get_contests)
            .service(get_contest_id)
            .service(get_contest_id_rank_list)
    })
    .bind(("127.0.0.1", 12345))?
    .run()
    .await
}


//   cargo run -- -c tests/cases/03_01_job_list.config.json

//    cargo run -- -c tests/cases/04_01_user_support.config.json

//    cargo test --test basic_requirements -- --test-threads=1

//    cargo test --test advanced_requirements -- --test-threads=1
