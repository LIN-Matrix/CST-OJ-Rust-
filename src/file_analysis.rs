use crate::{
    ALL_USER_SUBMIT_ARRAY, API_ARRAY, CONTEST_ALL_USER_SUBMIT_ARRAY, CONTEST_ARRAY,
    CONTEST_HIGHEST_USER_SUBMIT_ARRAY, HIGHEST_USER_SUBMIT_ARRAY, PROBLEM_ID_ARRAY,
    QUEUEING_RESPONSE_ARRAY, RECORD_CONTEST_USER_PROBLEM_ARRAY, RECORD_USER_PROBLEM_ARRAY,
    RESPONSE_ARRAY, USER_ARRAY, USER_ID_ARRAY, USER_NAME_ARRAY,
};
use std::collections::VecDeque;
use crate::api_analysis;
use std::io::BufReader;
use std::io::BufWriter;

/*************************************************************************
【函数名称】                file_flush
【函数功能】                进行--flush命令下的保留数据清除
【参数】                   无
【返回值】                 Result<()>类型，标志清理是否成功执行
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-7-12
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub fn file_flush() -> std::io::Result<()> {
    std::fs::create_dir_all("src/save")?;

    //1
    std::fs::File::create("src/save/api_array.json")?;
    std::fs::write("src/save/api_array.json", "[]")?;

    //2
    std::fs::File::create("src/save/contest_array.json")?;
    std::fs::write("src/save/contest_array.json", "[]")?;

    //3
    std::fs::File::create("src/save/response_array.json")?;
    std::fs::write("src/save/response_array.json", "[]")?;

    //4
    std::fs::File::create("src/save/queueing_array.json")?;
    std::fs::write("src/save/queueing_array.json", "[]")?;

    //5
    std::fs::File::create("src/save/user_array.json")?;
    std::fs::write("src/save/user_array.json", "[]")?;

    //6
    std::fs::File::create("src/save/user_id_array.json")?;
    std::fs::write("src/save/user_id_array.json", "[]")?;

    //7
    std::fs::File::create("src/save/user_name_array.json")?;
    std::fs::write("src/save/user_name_array.json", "[]")?;

    //8
    std::fs::File::create("src/save/problem_id_array.json")?;
    std::fs::write("src/save/problem_id_array.json", "[]")?;

    //9
    std::fs::File::create("src/save/record_user_problem_array.json")?;
    std::fs::write("src/save/record_user_problem_array.json", "[]")?;

    //10
    std::fs::File::create("src/save/all_user_submit_array.json")?;
    std::fs::write("src/save/all_user_submit_array.json", "[]")?;

    //11
    std::fs::File::create("src/save/highest_user_submit_array.json")?;
    std::fs::write("src/save/highest_user_submit_array.json", "[]")?;

    //12
    std::fs::File::create("src/save/record_contest_user_problem_array.json")?;
    std::fs::write("src/save/record_contest_user_problem_array.json", "[]")?;

    //13
    std::fs::File::create("src/save/contest_highest_user_submit_array.json")?;
    std::fs::write("src/save/contest_highest_user_submit_array.json", "[]")?;

    //14
    std::fs::File::create("src/save/contest_all_user_submit_array.json")?;
    std::fs::write("src/save/contest_all_user_submit_array.json", "[]")?;

    Ok(())
}

/*************************************************************************
【函数名称】                file_read
【函数功能】                进行每次get指令前的全局变量文件的读取
【参数】                   无
【返回值】                 Result<()>类型，标志读取是否成功执行
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-7-12
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub fn file_read() -> std::io::Result<()> {
    //1
    let api_file = std::fs::File::open("src/save/api_array.json")?;
    let api_from: VecDeque<api_analysis::APIPostJob> =
        serde_json::from_reader(BufReader::new(api_file))?;
    let mut api_to = API_ARRAY.lock().unwrap();
    for a in api_from {
        api_to.push_back(a);
    }

    //2
    let response_file = std::fs::File::open("src/save/response_array.json")?;
    let response_from: VecDeque<api_analysis::APIPostResponse> =
        serde_json::from_reader(BufReader::new(response_file))?;
    let mut response_to = RESPONSE_ARRAY.lock().unwrap();
    for r in response_from {
        response_to.push_back(r);
    }

    //3
    let contest_file = std::fs::File::open("src/save/contest_array.json")?;
    let contest_from: Vec<api_analysis::PostContest> =
        serde_json::from_reader(BufReader::new(contest_file))?;
    let mut contest_to = CONTEST_ARRAY.lock().unwrap();
    for c in contest_from {
        contest_to.push(c);
    }

    //4
    let new_file = std::fs::File::open("src/save/queueing_array.json")?;
    let new_from: Vec<api_analysis::APIPostResponse> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = QUEUEING_RESPONSE_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push_back(n);
    }

    //5
    let new_file = std::fs::File::open("src/save/user_array.json")?;
    let new_from: Vec<api_analysis::APIPostUsers> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = USER_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //6
    let new_file = std::fs::File::open("src/save/user_id_array.json")?;
    let new_from: Vec<usize> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = USER_ID_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //7
    let new_file = std::fs::File::open("src/save/user_name_array.json")?;
    let new_from: Vec<String> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = USER_NAME_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //8
    let new_file = std::fs::File::open("src/save/problem_id_array.json")?;
    let new_from: Vec<usize> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = PROBLEM_ID_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //9
    let new_file = std::fs::File::open("src/save/record_user_problem_array.json")?;
    let new_from: Vec<api_analysis::RecordUserProblem> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = RECORD_USER_PROBLEM_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //10
    let new_file = std::fs::File::open("src/save/all_user_submit_array.json")?;
    let new_from: Vec<usize> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = ALL_USER_SUBMIT_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //11
    let new_file = std::fs::File::open("src/save/highest_user_submit_array.json")?;
    let new_from: Vec<usize> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //12
    let new_file = std::fs::File::open("src/save/record_contest_user_problem_array.json")?;
    let new_from: Vec<api_analysis::RecordContestUserProblem> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = RECORD_CONTEST_USER_PROBLEM_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //13
    let new_file = std::fs::File::open("src/save/contest_highest_user_submit_array.json")?;
    let new_from: Vec<(usize,usize)> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to = CONTEST_HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    //14
    let new_file = std::fs::File::open("src/save/contest_all_user_submit_array.json")?;
    let new_from: Vec<(usize,usize)> =
        serde_json::from_reader(BufReader::new(new_file))?;
    let mut new_to =CONTEST_ALL_USER_SUBMIT_ARRAY.lock().unwrap();
    for n in new_from {
        new_to.push(n);
    }

    Ok(())
}

/*************************************************************************
【函数名称】                file_save
【函数功能】                进行每次post指令后的全局变量文件的保存
【参数】                   无
【返回值】                 Result<()>类型，标志保存是否成功执行
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-7-12
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub fn file_save() -> std::io::Result<()> {
    std::fs::create_dir_all("src/save")?;

    //1
    let api_file = std::fs::File::create("src/save/api_array.json")?;
    serde_json::to_writer_pretty(BufWriter::new(api_file), &API_ARRAY.lock().unwrap().clone())?;

    //2
    let response_file = std::fs::File::create("src/save/response_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(response_file),
        &RESPONSE_ARRAY.lock().unwrap().clone(),
    )?;

    //3
    let contest_file = std::fs::File::create("src/save/contest_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(contest_file),
        &CONTEST_ARRAY.lock().unwrap().clone(),
    )?;

    //4
    let new_file = std::fs::File::create("src/save/queueing_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &QUEUEING_RESPONSE_ARRAY.lock().unwrap().clone(),
    )?;

    //5
    let new_file = std::fs::File::create("src/save/user_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &USER_ARRAY.lock().unwrap().clone(),
    )?;

    //6
    let new_file = std::fs::File::create("src/save/user_id_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &USER_ID_ARRAY.lock().unwrap().clone(),
    )?;

    //7
    let new_file = std::fs::File::create("src/save/user_name_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &USER_NAME_ARRAY.lock().unwrap().clone(),
    )?;

    //8
    let new_file = std::fs::File::create("src/save/problem_id_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &PROBLEM_ID_ARRAY.lock().unwrap().clone(),
    )?;

    //9
    let new_file = std::fs::File::create("src/save/record_user_problem_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &RECORD_USER_PROBLEM_ARRAY.lock().unwrap().clone(),
    )?;

    //10
    let new_file = std::fs::File::create("src/save/all_user_submit_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &ALL_USER_SUBMIT_ARRAY.lock().unwrap().clone(),
    )?;

    //11
    let new_file = std::fs::File::create("src/save/highest_user_submit_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap().clone(),
    )?;

    //12
    let new_file = std::fs::File::create("src/save/record_contest_user_problem_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &RECORD_CONTEST_USER_PROBLEM_ARRAY.lock().unwrap().clone(),
    )?;

    //13
    let new_file = std::fs::File::create("src/save/contest_highest_user_submit_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &CONTEST_HIGHEST_USER_SUBMIT_ARRAY.lock().unwrap().clone(),
    )?;

    //14
    let new_file = std::fs::File::create("src/save/contest_all_user_submit_array.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(new_file),
        &CONTEST_ALL_USER_SUBMIT_ARRAY.lock().unwrap().clone(),
    )?;

    Ok(())
}