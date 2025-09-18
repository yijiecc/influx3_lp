use influx3_lp::Influx3Lp;
    
#[test]
fn test_lp_macro() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
        pub co: i32,
        pub weather: String,
        #[influx3_lp(timestamp)]
        pub timestamp: i64,
        #[influx3_lp(tag)]
        pub room: String,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        co: 0,
        weather: String::from("sunny"),
        timestamp: 1735545600,
        room: String::from("Kitchen"),
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home,room=Kitchen temp=21,hum=35.9,co=0i,weather=\"sunny\" 1735545600");
}

#[test]
fn test_multiple_tags() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
        #[influx3_lp(tag)]
        pub room: String,
        #[influx3_lp(tag)]
        pub city: String,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        room: String::from("Kitchen"),
        city: String::from("New York"),
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home,room=Kitchen,city=New\\ York temp=21,hum=35.9");
}

#[test]
fn test_empty_tags() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home temp=21,hum=35.9");
}

#[test]
fn test_has_at_least_one_field() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/field_count.rs");
}

#[test]
fn test_string_field() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
        pub date: String,
        pub time: String,
        #[influx3_lp(tag)]
        pub room: String,
        #[influx3_lp(tag)]
        pub city: String,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        date: String::from("2025 09 18"),
        time: String::from("12/12/12"),
        room: String::from("Kitchen"),
        city: String::from("New York"),
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home,room=Kitchen,city=New\\ York temp=21,hum=35.9,date=\"2025 09 18\",time=\"12/12/12\"");
}

#[test]
fn test_string_limit() {
    let maximum_string = "A".repeat(64 * 1024);

    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
        pub content: String,
        #[influx3_lp(tag)]
        pub room: String,
        #[influx3_lp(timestamp)]
        pub timestamp: i64,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        room: String::from("Kitchen"),
        content: maximum_string.clone(),
        timestamp: 1735545600,
    };

    let serialized = data.to_lp();
    let expected = format!("home,room=Kitchen temp=21,hum=35.9,content=\"{}\" 1735545600", maximum_string);
    assert_eq!(serialized, expected);
}

#[test]
#[should_panic(expected = "Length of string field value has a limit of 64K")]
fn test_string_limit_error() {
    let exceeded_string = "A".repeat(64 * 1024 + 1);

    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
        pub content: String,
        #[influx3_lp(tag)]
        pub room: String,
        #[influx3_lp(timestamp)]
        pub timestamp: i64,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        room: String::from("Kitchen"),
        content: exceeded_string.clone(),
        timestamp: 1735545600,
    };

    let serialized = data.to_lp();
    let expected = format!("home,room=Kitchen temp=21,hum=35.9,content=\"{}\" 1735545600", exceeded_string);
    assert_eq!(serialized, expected);
}

#[test]
fn test_special_charactors() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "special data")]
    struct SpecialData {
        pub f1: String,
        pub f2: String,
        pub f3: String,
        pub f4: String,
        #[influx3_lp(tag)]
        pub t1: String,
        #[influx3_lp(tag)]
        pub t2: String,
        #[influx3_lp(tag)]
        pub t3: String,
    }
    
    let data = SpecialData {
        f1: String::from("with,comma"),
        f2: String::from("with equal ="),
        f3: String::from(" with space "),
        f4: String::from("double quote \" and back slash\\"),
        t1: String::from("with,comma"),
        t2: String::from("with equal ="),
        t3: String::from(" with space "),
    };

    let serialized = data.to_lp();
    let expected = format!("special\\ data,t1=with\\,comma,t2=with\\ equal\\ \\=,t3=\\ with\\ space\\  f1=\"with,comma\",f2=\"with equal =\",f3=\" with space \",f4=\"double quote \\\" and back slash\\\\\"");
    println!("{}", expected);
    assert_eq!(serialized, expected);    
}

#[test]
fn test_empty_timestamp() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home temp=21,hum=35.9");
}

#[test]
fn test_integer_and_uinteger() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub int_data: i64,
        pub uint_data: u64,
    }
    
    let data = SensorData {
        int_data: 123456,
        uint_data: 123456,
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home int_data=123456i,uint_data=123456u");
}

#[test]
fn test_boolean_field() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub int_data: i64,
        pub uint_data: u64,
        pub bool_data: bool,
        pub bool_data_2: bool,
    }
    
    let data = SensorData {
        int_data: 123456,
        uint_data: 123456,
        bool_data: false,
        bool_data_2: true,
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home int_data=123456i,uint_data=123456u,bool_data=false,bool_data_2=true");
}

#[test]
fn test_optional_field_and_tag() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub int_data: Option<i64>,
        pub uint_data: Option<u64>,
        pub bool_data: Option<bool>,
        pub bool_data_2: Option<bool>,
        #[influx3_lp(tag)]
        pub tag1: Option<String>,
        #[influx3_lp(tag)]
        pub tag2: Option<String>,
    }
    
    let data = SensorData {
        int_data: Some(123456),
        uint_data: None,
        bool_data: Some(false),
        bool_data_2: None,
        tag1: Some(String::from("tag1")),
        tag2: None,
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home,tag1=tag1 int_data=123456i,bool_data=false");
}


#[test]
fn test_optional_timestamp() {
    #[derive(Influx3Lp)]
    #[influx3_lp(table_name = "home")]
    struct SensorData {
        pub temp: f32,
        pub hum: f64,
        #[influx3_lp(timestamp)]
        pub timestamp: Option<i64>,
    }
    
    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        timestamp: Some(123456),
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home temp=21,hum=35.9 123456");

    let data = SensorData {
        temp: 21.0,
        hum: 35.9,
        timestamp: None,
    };

    let serialized = data.to_lp();
    assert_eq!(serialized, 
               "home temp=21,hum=35.9");
}

