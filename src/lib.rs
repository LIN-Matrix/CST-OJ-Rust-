pub mod api_analysis;
pub mod config_analysis;
pub mod file_analysis;
pub mod trail_terminal;

#[macro_use]
extern crate lazy_static;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

lazy_static! {
    /*************************************************************************
    【静态变量功能】            存储所有提交的任务记录，VecDeque中为用户提交的代码信息等
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref API_ARRAY: Arc<Mutex<VecDeque<api_analysis::APIPostJob>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    /*************************************************************************
    【静态变量功能】            存储所有返回的响应记录，VecDeque中为处理后的对每个任务的评测结果
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref RESPONSE_ARRAY: Arc<Mutex<VecDeque<api_analysis::APIPostResponse>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的比赛数据，Vec中为目前所有的比赛信息
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref CONTEST_ARRAY: Arc<Mutex<Vec<api_analysis::PostContest>>> =
        Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有提交的任务的等待队列
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref QUEUEING_RESPONSE_ARRAY: Arc<Mutex<VecDeque<api_analysis::APIPostResponse>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的用户信息，包括用户ID和用户名
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref USER_ARRAY: Arc<Mutex<Vec<api_analysis::APIPostUsers>>> =
        Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的用户ID，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref USER_ID_ARRAY: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的用户姓名，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref USER_NAME_ARRAY: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的题目ID，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref PROBLEM_ID_ARRAY: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的用户与题目的一一对应的数据，目的是便于排行时数据分析和查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref RECORD_USER_PROBLEM_ARRAY: Arc<Mutex<Vec<api_analysis::RecordUserProblem>>> =
        Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的用户提交信息，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref ALL_USER_SUBMIT_ARRAY: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的用户最高分的提交信息，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref HIGHEST_USER_SUBMIT_ARRAY: Arc<Mutex<Vec<usize>>> =
        Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的某一比赛的用户提交信息，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref RECORD_CONTEST_USER_PROBLEM_ARRAY: Arc<Mutex<Vec<api_analysis::RecordContestUserProblem>>> =
        Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的某一比赛的用户最高分的提交信息，目的是便于查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref CONTEST_HIGHEST_USER_SUBMIT_ARRAY: Arc<Mutex<Vec<(usize, usize)>>> =
        Arc::new(Mutex::new(Vec::new()));
    /*************************************************************************
    【静态变量功能】            存储所有的某一比赛的用户与题目的一一对应的数据，目的是便于排行时数据分析和查找
    【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
    【更改记录】                2022-9-8 由林奕辰增加注释
    *************************************************************************/
    pub static ref CONTEST_ALL_USER_SUBMIT_ARRAY: Arc<Mutex<Vec<(usize, usize)>>> =
        Arc::new(Mutex::new(Vec::new()));
}
