//! # Suwon mate admin tool
//!
//! `suwon_mate_admin_tool`은 수원 메이트 앱을 위한 DB를 생성한다.
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use clap::Parser;
use serde_json::{json, Value};

/// 수원메이트용 DB제작 프로그램
///
/// 수원 메이트 앱 용으로 사용될 json형태의 DB 파일을 제작할 수 있습니다.
#[derive(Parser)]
#[command(author)]
pub struct ProgramArgument {
    /// 개설 강좌 조회 DB 파일
    #[arg(short, long)]
    pub open_class_file: String,
    /// 강의 계획서 DB 파일
    #[arg(short, long)]
    pub class_todo_file: String,
    /// DB에 기입할 최신 앱 버전
    #[arg(short, long, default_value_t = String::from("1.0"))]
    pub app_version: String,
    /// DB에 기입할 DB 버전
    #[arg(short, long)]
    pub db_version: String,
    /// DB에 기입할 레거시 앱 버전
    #[arg(short, long, default_value_t = String::from("1.0"))]
    pub legacy_app_version: String,
}


#[derive(PartialEq, Debug)]
/// 강의계획서로부터 가져온 특정 과목의 학부, 학과, 이메일 주소를 가진 구조체이다.
pub struct ClassTodo<'todo_class> {
    /// 과목 수강 대상자의 학부
    pub department: &'todo_class Value,
    /// 과목 수강 대상자의 학과
    pub major: &'todo_class Value,
    /// 강의자의 이메일 주소
    pub email: &'todo_class Value,
    /// 강의자의 휴대전화 번호
    pub phone: &'todo_class Value,
}

impl<'todo_class> ClassTodo<'todo_class> {
    /// 특정 과목의 학부, 학과, 이메일 주소 포함한 [Department]를 생성한다.
    ///
    /// ## Examples
    /// ```
    /// use serde_json::Value;
    /// use suwon_mate_admin_tool::ClassTodo;
    /// let department_info = ClassTodo::new(&Value::String("컴퓨터학부".to_string()), &Value::String("컴퓨터학과".to_string()), &Value::String("sun30812@naver.com".to_string()), &Value::String("010-0000-0000".to_string()));
    /// ```
    pub fn new(
        department: &'todo_class Value,
        major: &'todo_class Value,
        email: &'todo_class Value,
        phone: &'todo_class Value,
    ) -> Self {
        Self {
            department,
            major,
            email,
            phone,
        }
    }
    /// **강의 계획서에서** 과목의 정보가 포함된 목록인 `subjects`로부터 특정 과목의 학부, 학과, 이메일 주소, 전화번호를 알아낼 때 사용할 수 있는 메서드
    ///
    /// ## Examples
    /// ```
    /// use std::fs::File;
    /// use std::io::Read;
    /// use serde_json::Value;
    /// use suwon_mate_admin_tool::ClassTodo;
    /// let mut dummy_open_class_file =
    ///             File::open("sample/sample_todo_class.json").expect("Sample파일을 찾을 수 없습니다.");
    ///         let mut dummy_open_class_data = String::new();
    ///         dummy_open_class_file
    ///             .read_to_string(&mut dummy_open_class_data)
    ///             .expect("Sample파일을 읽을 수 없습니다.");
    ///         let subjects: Value = serde_json::from_str(&dummy_open_class_data).expect("JSON 파싱 실패");
    ///         let subjects = subjects["estbLectDtaiList"]
    ///             .as_array()
    ///             .expect("JSON 파싱 실패(2단계)");
    ///         assert_eq!(
    ///             ClassTodo::new(&Value::String("경영학부".to_string()), &Value::Null, &Value::String("test@suwon.ac.kr".to_string()), &Value::String("010-0000-0000".to_string())),
    ///             ClassTodo::get_department_info(subjects, "11416", "038")
    ///         )
    ///```
    pub fn get_department_info(
        subjects: &'todo_class Vec<Value>,
        subject_code: &str,
        dicl_number: &str,
    ) -> Self {
        for subject in subjects.iter() {
            if format!(
                "{}-{}",
                subject["subjtCd"].as_str().unwrap_or(""),
                subject["diclNo"].as_str().unwrap_or("")
            ) == format!("{}-{}", subject_code, dicl_number)
            {
                return Self::new(
                    &subject["estbDpmjNm"],
                    &subject["estbMjorNm"],
                    &subject["email"],
                    &subject["mpno"],
                );
            }
        }
        Self::new(&Value::Null, &Value::Null, &Value::Null, &Value::Null)
    }
}

/// 지정된 파일을 읽고 쓰는 작업을 진행하는 메서드
///
/// `program_args`로부터 필요한 인자값을 받아서 파일을 읽고 작업 후 파일을 쓰는 작업을 진행한다.
/// ## Errors
/// * 인자값으로 주어진 파일이 존재하지 않는 경우
/// * 표준 IO가 정상적으로 작동하지 않는 경우
/// ## Panics
/// 파일의 쓰기권한이 부여되지 않은 경우 해당 메서드는 호출될 수 없다.
pub fn file_process(program_args: ProgramArgument) -> Result<(), Box<dyn Error>> {
    let mut open_class_file = File::open(program_args.open_class_file)?;
    let mut class_todo_file = File::open(program_args.class_todo_file)?;
    let mut open_class_content = String::new();
    let mut class_todo_content = String::new();
    open_class_file.read_to_string(&mut open_class_content)?;
    class_todo_file.read_to_string(&mut class_todo_content)?;
    let mut db_file = File::create(format!("result_{}.json", program_args.db_version))
        .unwrap_or_else(|error| {
            println!(
                "다음과 같은 이유로 DB 파일 생성에 실패하였습니다: {}",
                error
            );
            std::process::exit(1);
        });
    db_file
        .write_all(
            make_db_content(
                &open_class_content,
                &class_todo_content,
                &program_args.app_version,
                &program_args.db_version,
                open_class_content == class_todo_content,
            )
                .unwrap_or_else(|error| {
                    println!(
                        "DB 내용 생성 과정에서 다음과 같은 오류가 발생되었습니다: {}",
                        error
                    );
                    std::process::exit(1);
                })
                .as_bytes(),
        )
        .expect("DB파일 쓰기 실패");
    println!(
        "작업이 완료되었습니다. result_{}.json파일로 저장되었습니다.",
        program_args.db_version.clone()
    );
    Ok(())
}

/// DB의 내용물을 만드는 메서드
///
/// 제공된 두 파일의 내용과 인자값을 바탕으로 최종 DB파일을 생성하는 메서드이다.
/// 제공된 파일에서 필요한 부분들만 합쳐서 진행되며, 만일 필요한 부분에 대한 정보가 제공된 파일에 존재하지 않는 경우
/// `null`로 기록된다.
/// `quick_mode`를 통해 생성한 DB는 빠른 개설 강좌 조회용 DB임을 명시할 수 있다. 이 경우 `estbLectDtaiList_quick`라는
/// 키 값을 통해 과목 정보에 접근할 수 있다.
///
/// ## Errors
/// 제공된 파일의 내용을 기반으로 JSON해독이 불가능 한 경우 오류가 발생한다.
pub fn make_db_content<'make_db>(
    open_class_content: &'make_db str,
    class_todo_content: &'make_db str,
    latest_app_version: &'make_db str,
    db_version: &'make_db str,
    quick_mode: bool,
) -> Result<String, Box<dyn Error>> {
    let open_class_data: Value = serde_json::from_str(open_class_content)?;
    let class_todo_data: Value = serde_json::from_str(class_todo_content)?;
    let departments = class_todo_data["estbLectDtaiList"]
        .as_array()
        .unwrap_or_else(|| {
            println!("강의 계획서 DB로부터 학부 목록을 가져오는데 문제가 발생하였습니다.");
            std::process::exit(1);
        });
    let mut departments_set = HashSet::new();
    for department in departments.iter() {
        departments_set.insert(department["estbDpmjNm"].as_str().unwrap_or_else(|| {
            println!("계획서 파일에서 누락된 학부가 존재합니다.");
            ""
        }));
    }
    let mut departments_map: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut subject_map: HashMap<String, Vec<Value>> = HashMap::new();
    let mut contact_map: HashMap<String, HashMap<String, Value>> = HashMap::new();
    for department in departments_set.iter() {
        subject_map.insert(department.parse()?, vec![]);
        contact_map.insert(department.parse()?, HashMap::new());
    }
    let open_subjects = open_class_data["estbLectDtaiList"]
        .as_array()
        .unwrap_or_else(|| {
            println!("개설 강죄 조회 DB로부터 과목정보를 가져오는데 문제가 발생하였습니다.");
            std::process::exit(1);
        });
    let todo_subjects = class_todo_data["estbLectDtaiList"]
        .as_array()
        .unwrap_or_else(|| {
            println!("강의 계획서 DB로부터 과목 정보를 가져오는데 문제가 발생하였습니다.");
            std::process::exit(1);
        });
    for subject in open_subjects.iter() {
        let temp = ClassTodo::get_department_info(
            todo_subjects,
            subject["subjtCd"].as_str().unwrap_or(""),
            subject["diclNo"].as_str().unwrap_or(""),
        );
        if !temp.major.is_null() {
            if !departments_map.contains_key(temp.department.as_str().unwrap()) {
                departments_map.insert(temp.department.as_str().unwrap(), HashSet::new());
            }
            departments_map
                .get_mut(temp.department.as_str().unwrap())
                .unwrap()
                .insert(temp.major.as_str().unwrap());
        }
        if let Some(subject_map) = subject_map.get_mut(temp.department.as_str().unwrap()) {
            subject_map.push(json!({
                "trgtGrdeCd": subject["trgtGrdeCd"],
                "subjtNm": subject["subjtNm"],
                "ltrPrfsNm" : subject["ltrPrfsNm"],
                "deptNm" : subject["deptNm"],
                "facDvnm" : subject["facDvnm"],
                "timtSmryCn" : subject["timtSmryCn"],
                "lssnLangNm" : subject["lssnLangNm"],
                "subjtCd": subject["subjtCd"],
                "diclNo" : subject["diclNo"],
                "subjtEstbYear" : subject["subjtEstbYear"],
                "point" : subject["point"],
                "cltTerrNm" : subject["cltTerrNm"],
                "sexCdNm" : subject["sexCdNm"],
                "hffcStatNm" : subject["hffcStatNm"],
                "clsfNm": subject["clsfNm"],
                "capprTypeNm" : subject["capprTypeNm"],
                "estbDpmjNm": temp.department,
                "estbMjorNm": temp.major,
            }));
        } else {
            println!(
                "주의: 분류에 실패한 학부 및 학과가 존재합니다. ({})",
                temp.department.as_str().unwrap()
            )
        }
        if let Some(contact_map) = contact_map.get_mut(temp.department.as_str().unwrap()) {
            match subject["ltrPrfsNm"].as_str() {
                None => {}
                Some(name) => {
                    contact_map.insert(name.to_string(), json!({
            "email": temp.email,
            "mpno": temp.phone
            }));
                }
            }
        }
    }
    let result = json!({
        if quick_mode {"departments_quick"} else {"departments"}: departments_map,
        if quick_mode {"estbLectDtaiList_quick"} else {"estbLectDtaiList"}: subject_map,
    "contacts": contact_map,
        "version": {
            "app_ver": latest_app_version,
            "db_ver": db_version,
            "legacy_app_ver": "0.0"
        }

    });
    Ok(result.to_string())
}
