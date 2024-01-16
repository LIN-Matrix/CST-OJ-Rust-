use crate::api_analysis::RankListScoringRule::Latest;
use chrono;
use serde::{Deserialize, Serialize};
use serde_json;

/*************************************************************************
【结构体名称】              APIErr
【结构体功能】              记录错误的结果和信息
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIErr {
    pub code: usize,
    pub reason: String,
    pub message: String,
}

impl APIErr {
    pub fn new() -> APIErr {
        APIErr {
            code: 0,
            reason: String::new(),
            message: String::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              APIPostJob
【结构体功能】              记录提交的任务的信息，包括所含代码，对应题目，比赛编号等等
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIPostJob {
    pub source_code: String,
    pub language: String,
    pub user_id: usize,
    pub contest_id: usize,
    pub problem_id: usize,
}

impl APIPostJob {
    pub fn new() -> APIPostJob {
        APIPostJob {
            source_code: String::new(),
            language: String::new(),
            user_id: 0,
            contest_id: 0,
            problem_id: 0,
        }
    }
}

/*************************************************************************
【结构体名称】              APIPostUsers
【结构体功能】              记录提交的用户信息，包括ID（可选）和名字
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIPostUsers {
    pub id: Option<usize>,
    pub name: String,
}

impl APIPostUsers {
    pub fn new() -> APIPostUsers {
        APIPostUsers {
            id: None,
            name: String::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              APIPostJob
【结构体功能】              后端服务器对提交任务的反馈，包括任务编号，创建、更新时间，
                         提交代码，当前状态，任务结果，成绩，所有情况结果等
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIPostResponse {
    pub id: usize,
    pub created_time: String,
    pub updated_time: String,
    pub submission: APIPostJob,
    pub state: APIPostResponseState,
    pub result: APIPostResponseResult,
    pub score: f32,
    pub cases: Vec<APIPostResponseCase>,
}

impl APIPostResponse {
    pub fn new() -> APIPostResponse {
        APIPostResponse {
            id: 0,
            created_time: chrono::prelude::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            updated_time: chrono::prelude::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            submission: APIPostJob::new(),
            state: APIPostResponseState::Queueing,
            result: APIPostResponseResult::Waiting,
            score: 0.0,
            cases: Vec::new(),
        }
    }
}

//枚举，对应任务的状态
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum APIPostResponseState {
    Queueing,
    Running,
    Finished,
    Canceled,
}


//枚举，对应任务和测试点的结果
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum APIPostResponseResult {
    Waiting,
    Running,
    Accepted,
    #[serde(rename = "Compilation Error")]
    CompilationError,
    #[serde(rename = "Compilation Success")]
    CompilationSuccess,
    #[serde(rename = "Wrong Answer")]
    WrongAnswer,
    #[serde(rename = "Runtime Error")]
    RuntimeError,
    #[serde(rename = "Time Limit Exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    MemoryLimitExceeded,
    #[serde(rename = "System Error")]
    SystemError,
    #[serde(rename = "SPJ Error")]
    SPJError,
    Skipped,
}


impl APIPostResponseResult {
    pub fn show_str(s: &Self) -> &str {
        match s {
            APIPostResponseResult::Waiting => "Waiting",
            APIPostResponseResult::Accepted => "Accepted",
            APIPostResponseResult::TimeLimitExceeded => "TimeLimitExceeded",
            APIPostResponseResult::Running => "Running",
            APIPostResponseResult::RuntimeError => "RuntimeError",
            APIPostResponseResult::WrongAnswer => "WrongAnswer",
            APIPostResponseResult::SPJError => "SPJError",
            APIPostResponseResult::CompilationSuccess => "CompilationSuccess",
            APIPostResponseResult::CompilationError => "CompilationError",
            APIPostResponseResult::Skipped => "Skipped",
            APIPostResponseResult::MemoryLimitExceeded => "MemoryLimitExceeded",
            APIPostResponseResult::SystemError => "SystemError",
        }
    }
}

/*************************************************************************
【结构体名称】              APIPostResponseCase
【结构体功能】              记录每个测试点的详细信息，包括编号、结果、用时、内存等
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct APIPostResponseCase {
    pub id: usize,
    pub result: APIPostResponseResult,
    pub time: usize,
    pub memory: usize,
    pub info: String,
}

impl APIPostResponseCase {
    pub fn new() -> APIPostResponseCase {
        APIPostResponseCase {
            id: 0,
            result: APIPostResponseResult::Waiting,
            time: 0,
            memory: 0,
            info: String::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              PostContest
【结构体功能】              记录比赛的信息，包括编号、名称、起止时间、所含题目等
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostContest {
    pub id: Option<usize>,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<usize>,
    pub user_ids: Vec<usize>,
    pub submission_limit: usize,
}

impl PostContest {
    pub fn new() -> PostContest {
        PostContest {
            id: None,
            name: String::new(),
            from: String::new(),
            to: String::new(),
            problem_ids: Vec::new(),
            user_ids: Vec::new(),
            submission_limit: 0,
        }
    }
}

/*************************************************************************
【结构体名称】              GetJobInfo
【结构体功能】              记录需要获取的任务的信息，这些信息用于筛选目标的任务列
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetJobInfo {
    pub user_id: Option<usize>,
    pub user_name: Option<String>,
    pub contest_id: Option<usize>,
    pub problem_id: Option<usize>,
    pub language: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub state: Option<APIPostResponseState>,
    pub result: Option<APIPostResponseResult>,
}

impl GetJobInfo {
    pub fn new() -> GetJobInfo {
        GetJobInfo {
            user_id: None,
            user_name: None,
            contest_id: None,
            problem_id: None,
            language: None,
            from: None,
            to: None,
            state: None,
            result: None,
        }
    }
}

/*************************************************************************
【结构体名称】              GetContestIDRankList
【结构体功能】              获取所需的排序方式和打破平局的方法，None则采取默认方法
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetContestIDRankList {
    pub scoring_rule: Option<RankListScoringRule>,
    pub tie_breaker: Option<RankListTieBreaker>,
}

impl GetContestIDRankList {
    pub fn new() -> GetContestIDRankList {
        GetContestIDRankList {
            scoring_rule: Some(RankListScoringRule::Latest),
            tie_breaker: None,
        }
    }
}

//枚举，取最后一次提交还是最高分的一次提交
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RankListScoringRule {
    #[serde(rename = "latest")]
    Latest,
    #[serde(rename = "highest")]
    Highest,
}

//枚举，通过提交时间，提交次数还是用户id来打破平局
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RankListTieBreaker {
    #[serde(rename = "submission_time")]
    SubmissionTime,
    #[serde(rename = "submission_count")]
    SubmissionCount,
    #[serde(rename = "user_id")]
    UserID,
}

/*************************************************************************
【结构体名称】              UserRankResponse
【结构体功能】              记录用户的排名和每个题目的得分
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRankResponse {
    pub user: APIPostUsers,
    pub rank: usize,
    pub scores: Vec<f32>,
}

impl UserRankResponse {
    pub fn new() -> UserRankResponse {
        UserRankResponse {
            user: APIPostUsers::new(),
            rank: 1,
            scores: Vec::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              RecordUserProblem
【结构体功能】              记录相同用户和题目下的多次提交和正确满分提交的每个测试点时间，
                         最短时间列表可以方便于竞争得分的计算
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordUserProblem {
    pub user_id: usize,
    pub problem_id: usize,
    pub scores: Vec<f32>,
    pub min_time: Vec<usize>,
}

impl RecordUserProblem {
    pub fn new() -> RecordUserProblem {
        RecordUserProblem {
            user_id: 0,
            problem_id: 0,
            scores: Vec::new(),
            min_time: Vec::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              RecordContestUserProblem
【结构体功能】              记录相同用户和题目和比赛编号下的多次提交和正确满分提交的
                         每个测试点时间，最短时间列表可以方便于竞争得分的计算
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordContestUserProblem {
    pub contest_id: usize,
    pub user_id: usize,
    pub problem_id: usize,
    pub scores: Vec<f32>,
    pub min_time: Vec<usize>,
}

impl RecordContestUserProblem {
    pub fn new() -> RecordContestUserProblem {
        RecordContestUserProblem {
            contest_id: 0,
            user_id: 0,
            problem_id: 0,
            scores: Vec::new(),
            min_time: Vec::new(),
        }
    }
}


/*************************************************************************
【函数名称】                get_job_judge
【函数功能】                测试用以判断和筛选所需的任务,只有每一项都符合才会返回true
【参数】                   info:为筛选条件，json_job:为每个反馈信息，用以筛选
【返回值】                 bool类型，true为满足，false则不满足
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                 2022-9-8 由林奕辰增加注释
*************************************************************************/
pub fn get_job_judge(info: &GetJobInfo, json_job: &APIPostResponse) -> bool {
    if let Some(id) = &info.problem_id {
        if id != &json_job.submission.problem_id {
            return false;
        }
    }
    let create_time = chrono::NaiveDateTime::parse_from_str(
        json_job.created_time.as_str(),
        "%Y-%m-%dT%H:%M:%S%.3fZ",
    )
    .unwrap();
    if let Some(s) = &info.from {
        let from_time =
            chrono::NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        if create_time < from_time {
            return false;
        }
    }
    if let Some(s) = &info.to {
        let to_time =
            chrono::NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        if create_time > to_time {
            return false;
        }
    }
    if let Some(state) = &info.state {
        if state != &json_job.state {
            return false;
        }
    }
    if let Some(result) = &info.result {
        if result != &json_job.result {
            return false;
        }
    }
    if let Some(language) = &info.language {
        if language != &json_job.submission.language {
            return false;
        }
    }
    if let Some(user_id) = &info.user_id{
        if user_id != &json_job.submission.user_id{
            return false;
        }
    }
    if let Some(contest_id) = &info.contest_id{
        if contest_id != &json_job.submission.contest_id{
            return false;
        }
    }

    true
}


