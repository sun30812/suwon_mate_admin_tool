//! # Suwon mate admin tool
//!
//! `suwon_mate_admin_tool`은 수원 메이트 앱을 위한 DB를 생성한다.
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use serde_json::{json, Value};

/// 프로그램 인자값 모음
///
/// 프로그램에서 사용되는 인자값들을 나타내는 구조체이다.
pub struct ProgramArgument<'a> {
    /// 개설 강좌 조회 DB 파일 이름
    pub open_class_filename: &'a str,
    /// 강의 계획서 DB 파일 이름
    pub class_todo_filename: &'a str,
    /// DB에 기입할 최신 앱 버전
    pub latest_app_version: &'a str,
    /// DB에 기입할 DB 버전
    pub db_version: &'a str,
}

impl<'a> ProgramArgument<'a> {
    /// 프로그램으로부터 인자값을 받아 [ProgramArgument]에 맞게 변환한다.
    ///
    /// # Error
    /// 만일 필요로 하는 인자의 개수보다 적은 경우 오류를 발생시킨다.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use suwon_mate_admin_tool::ProgramArgument;
    /// let args: Vec<String> = vec!["1".to_string(), "sample_todo_class.json".to_string(), "3".to_string(), "4".to_string(), "5".to_string()];
    ///
    ///     let program_arguments = ProgramArgument::new(&args).unwrap_or_else(|error| {
    ///         println!(
    ///             "인자값을 불러올 때 다음과 같은 오류가 발생했습니다.: {}",
    ///             error
    ///         );
    ///         std::process::exit(1);
    ///     });
    /// println!("개설 강좌 조회 파일 이름은 {} 입니다.", program_arguments.open_class_filename)
    ///```
    pub fn new(args: &'a [String]) -> Result<Self, &'static str> {
        if args.len() < 5 {
            return Err(
                "해당 프로그램인 인자로 개설 과목 DB 파일과 강의 계획서 DB 파일을 필요로 합니다.",
            );
        }

        Ok(Self {
            open_class_filename: &args[1],
            class_todo_filename: &args[2],
            latest_app_version: &args[3],
            db_version: &args[4],
        })
    }
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
    let mut open_class_file = File::open(program_args.open_class_filename)?;
    let mut class_todo_file = File::open(program_args.class_todo_filename)?;
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
                program_args.latest_app_version,
                program_args.db_version,
                program_args.open_class_filename == program_args.class_todo_filename,
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
    for department in departments_set.iter() {
        departments_map.insert(department, HashSet::new());
    }
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
            contact_map.insert(subject["ltrPrfsNm"].as_str().unwrap_or("").to_string(), json!({
            "email": temp.email,
            "mpno": temp.phone
            }));
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
