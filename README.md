# 在线评测系统（Rust语言实现）

Eachann Lin  （THU-CST 暑假小学期 rust 班级）



## 作业要求

OJ（Online Judge，在线评测系统）是在课程教学、算法竞赛等场合中用于测试程序正确性的线上系统。用户可以通过友好的界面提交自己的源代码，评测系统在指定的环境中编译代码，使用特定的输入运行程序，并将输出与答案进行比对。

随着需求的不断演化，各类开源的 OJ 系统不断涌现（如 [Vijos](https://github.com/vijos/vj4)、[UOJ](https://github.com/vfleaking/uoj)、[NOJ](https://github.com/ZsgsDesign/NOJ)、[HUSTOJ](https://github.com/zhblue/hustoj)、[Hydro](https://github.com/hydro-dev/Hydro)、[DMOJ](https://github.com/DMOJ/online-judge)、[CMS](https://github.com/cms-dev/cms)）。它们各具特色，均有广泛的用户群。我使用 Rust 语言，基于给出的项目模板实现一个 OJ 系统。如果没有特殊说明，它应该在各个平台下都能正常工作。

其中本地的 CLI 前端在  /src/bin/cli/main.rs 中，本地的后端在 /src/bin/oj/main.rs 中，它们可在命令行下同时分别运行。

## 自动测试

本作业的基础要求和部分提高要求可使用 Cargo 进行自动化测试。运行 `cargo test --test basic_requirements -- --test-threads=1` 可测试基础要求，`cargo test --test advanced_requirements -- --test-threads=1` 可测试部分提高要求。

如果某个测试点运行失败，将会打印 `case [name] incorrect` 的提示（可能会有额外的 `timeout` 提示，可以忽略）。你可以使用 `cargo test test_name` 单独运行此测试，也可以在 `tests/cases` 目录下查看相应测试用例的内容，并按照文档的说明调试。

自动测试运行每个测试点后，会生成以下的文件：

* `[case_name].stdout/stderr`：OJ 程序的标准输出和标准错误。你可以在代码中添加打印语句，然后结合输出内容来调试代码。
* `[case_name].http`：测试过程中发送的 HTTP 请求和收到的响应。调试时，你可以先自己启动一个 OJ 服务端（`cargo run`），然后用 VSCode REST Client 来手动发送这些 HTTP 请求，并观察响应。

项目配置了持续集成（CI）用于帮助你测试。在推送你的改动后，可以在 GitLab 网页上查看 CI 结果和日志。同时，上述的文件也会被收集到对应任务的 artifacts 中，你可以在 GitLab 网页上下载并查看。
