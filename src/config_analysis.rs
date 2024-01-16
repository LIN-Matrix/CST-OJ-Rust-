use serde::{Deserialize, Serialize};
use serde_json;

/*************************************************************************
【结构体名称】              Config
【结构体功能】              记录比赛设置，包括服务器，题目列表，语言支持
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ConfigServer,
    pub problems: Vec<ConfigProblem>,
    pub languages: Vec<ConfigLanguage>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            server: ConfigServer::new(),
            problems: Vec::new(),
            languages: Vec::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              ConfigServer
【结构体功能】              服务器地址和端口
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigServer {
    pub bind_address: Option<String>,
    pub bind_port: Option<usize>,
}

impl ConfigServer {
    pub fn new() -> ConfigServer {
        ConfigServer {
            bind_address: None,
            bind_port: None,
        }
    }
}

/*************************************************************************
【结构体名称】              ConfigProblem
【结构体功能】              记录比赛题目列表，包括题目编号、特殊信息，测试点信息等
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigProblem {
    pub id: usize,
    pub name: String,
    pub r#type: ConfigProblemType,
    pub misc: Option<ConfigProblemMisc>,
    pub cases: Vec<ConfigProblemCase>,
}

impl ConfigProblem {
    pub fn new() -> ConfigProblem {
        ConfigProblem {
            id: 0,
            name: String::new(),
            r#type: ConfigProblemType::Standard,
            misc: None,
            cases: Vec::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              ConfigProblemMisc
【结构体功能】              题目特殊信息，包括打包测试、特殊评测、竞争得分
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigProblemMisc {
    pub packing: Option<Vec<Vec<usize>>>,
    pub special_judge: Option<Vec<String>>,
    pub dynamic_ranking_ratio: Option<f32>,
}

impl ConfigProblemMisc {
    pub fn new() -> ConfigProblemMisc {
        ConfigProblemMisc {
            packing: None,
            special_judge: None,
            dynamic_ranking_ratio: None,
        }
    }
}

/*************************************************************************
【结构体名称】              ConfigLanguage
【结构体功能】              包括比赛的答案文件，命令行参数
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigLanguage {
    pub name: String,
    pub file_name: String,
    pub command: Vec<String>,
}

impl ConfigLanguage {
    pub fn new() -> ConfigLanguage {
        ConfigLanguage {
            name: String::new(),
            file_name: String::new(),
            command: Vec::new(),
        }
    }
}

/*************************************************************************
【结构体名称】              ConfigProblemCase
【结构体功能】              记录每个测试点信息，包括分数，输入文件，时间、内存限制等
【开发者及日期】            林奕辰(lin-yc21@mails.tsinghua.edu.cn) 2022-9-2
【更改记录】                2022-9-8 由林奕辰增加注释
*************************************************************************/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigProblemCase {
    pub score: f32,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: u64,
    pub memory_limit: u64,
}

impl ConfigProblemCase {
    pub fn new() -> ConfigProblemCase {
        ConfigProblemCase {
            score: 0.0,
            input_file: String::new(),
            answer_file: String::new(),
            time_limit: 0,
            memory_limit: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigProblemType {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "strict")]
    Strict,
    #[serde(rename = "spj")]
    SPJ,
    #[serde(rename = "dynamic_ranking")]
    DynamicRanking,
}

//该函数用于消除文件末尾的\n和文件每行末尾的\r、“ ”等
pub fn deal_string_standard(mut target: String) -> String {
    let mut slice_vec = Vec::new();
    let mut last_pos = 0;
    let mut target_chars: Vec<_> = target.clone().chars().collect();
    loop {
        match target.pop() {
            Some(s) => {
                if s == '\n' {
                    target_chars.pop();
                    continue;
                } else {
                    target.push(s);
                    break;
                }
            }
            None => break,
        }
    }
    for new_pos in 0..target.len() {
        if target_chars[new_pos] == '\n' {
            let mut single_slice = target[last_pos..new_pos].to_string();
            last_pos = new_pos;
            loop {
                match single_slice.pop() {
                    Some(s) => {
                        if s == ' ' || s == '\r' {
                            continue;
                        } else {
                            single_slice.push(s);
                            break;
                        }
                    }
                    None => break,
                }
            }
            slice_vec.push(single_slice);
        }
    }
    let mut single_slice = target[last_pos..target.len()].to_string();
    loop {
        match single_slice.pop() {
            Some(s) => {
                if s == ' ' || s == '\r' {
                    continue;
                } else {
                    single_slice.push(s);
                    break;
                }
            }
            None => break,
        }
    }
    slice_vec.push(single_slice);

    target = String::new();
    for single_slice in slice_vec {
        target += single_slice.as_str();
    }
    target
}
