mod data;

extern crate rand;
extern crate num_cpus;
extern crate secp256k1;

use std::{collections::HashSet, fs::{OpenOptions, File}, time::{Instant, Duration}, io::{BufRead, BufReader, Write}, path::Path, io};
use std::{
    io::{stdout},
};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::Sender;
use rand::Rng;
use sv::util::{hash160, sha256d};

use base58::ToBase58;
use rustils::parse::boolean::string_to_bool;
use secp256k1::{PublicKey, Secp256k1, SecretKey, All};
use tokio::task;

//Список для рандом
const HEX: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"];
const FILE_CONFIG: &str = "confPazl.txt";
const BACKSPACE: char = 8u8 as char;

#[tokio::main]
async fn main() {
    let count_cpu = num_cpus::get();
    //Чтение настроек, и если их нет создадим
    //-----------------------------------------------------------------
    let conf = match lines_from_file(&FILE_CONFIG) {
        Ok(text) => { text }
        Err(_) => {
            add_v_file(&FILE_CONFIG, data::get_conf_text().to_string());
            lines_from_file(&FILE_CONFIG).unwrap()
        }
    };

    let mut num_cores: i8 = first_word(&conf[0].to_string()).to_string().parse::<i8>().unwrap();
    let pazl: usize = first_word(&conf[1].to_string()).to_string().parse::<usize>().unwrap();
    let mut custom_digit = first_word(&conf[2].to_string()).to_string().parse::<String>().unwrap();
    let enum_start: usize = first_word(&conf[3].to_string()).to_string().parse::<usize>().unwrap();
    let enum_end: usize = first_word(&conf[4].to_string()).to_string().parse::<usize>().unwrap();
    let mut enum_all: u8 = first_word(&conf[5].to_string()).to_string().parse::<u8>().unwrap();
    let start_enum = first_word(&conf[7].to_string()).to_string();
    let end_enum = first_word(&conf[8].to_string()).to_string();
    let step = first_word(&conf[9].to_string()).to_string();
    let rnd_step = first_word(&conf[10].to_string()).to_string();
    //---------------------------------------------------------------------

    //если указана длинна пазла больше звёздочек , дорисуем звёздочек
    let cash = custom_digit.clone();
    let cd: Vec<&str> = cash.split(",").collect();
    if cd.len() <= pazl {
        for i in 0..pazl {
            if cd.get(i).is_none() {
                custom_digit.push_str(&*",*".to_string());
            }
        }
    }

    let rnd_step = string_to_bool(rnd_step.to_string());

    //если поставят полный перебор отключим последовательный и поставим на одно ядро
    if enum_start + enum_end >= pazl {
        enum_all = 0;
        //если выключен рандомный шаг
        if rnd_step == false {
            num_cores = 1;
        }
    }


    // Инфо блок
    // ---------------------------------------------------------------------
    println!("===============================");
    println!("FIND PAZL 66-160(17-40) v2.0.9");
    println!("===============================");

    println!("conf load:\n\
    CPU CORE:{num_cores}/{count_cpu}\n\
    HEX_END:{pazl}\n\
    CUSTOM_DIGIT\n{:?}\n\
    ENUMERATION_START:{enum_start}\n\
    ENUMERATION_END:{enum_end}\n\
    ENUMERATION STEP 1 ALL:{}\n\
    START_ENUMERATION:{start_enum}\n\
    END_ENUMERATION:{end_enum}\n\
    STEP:{step}\n\
    RAND STEP:{rnd_step}", custom_digit, string_to_bool(enum_all.clone().to_string()));
    // --------------------------------------------------------------------

    let file_content = match lines_from_file("puzzle.txt") {
        Ok(file) => { file }
        Err(_) => {
            let dockerfile = include_str!("puzzle.txt");
            add_v_file("puzzle.txt", dockerfile.to_string());
            lines_from_file("puzzle.txt").expect("kakoyto_pizdec")
        }
    };

    //хешируем
    let mut database = HashSet::new();
    for address in file_content.iter() {
        database.insert(address.to_string());
    }

    println!("\nADDRESS LOAD:{:?}", database.len());

    // Если 0 значит тест изменим на 1
    // -----------------------------------------------------------
    let mut bench = false;
    if num_cores == 0 {
        println!("--------------------------------");
        println!("        test mode 1 core");
        println!("--------------------------------");
        bench = true;
        num_cores = 1;
    }
    // ------------------------------------------------------------

    //переводим в число и обрезаем по длинне пересчета
    let dlinna_stert_range = if enum_start == 0 { start_enum.len() } else { enum_start };
    let start_enum = if start_enum != "0" { u128::from_str_radix(&*start_enum[0..dlinna_stert_range].to_string(), 16).unwrap() } else { 0 };
    let end_enum = if end_enum != "0" { u128::from_str_radix(&*end_enum[0..dlinna_stert_range].to_string(), 16).unwrap() } else { 0 };
    let step = u128::from_str_radix(&*step, 16).unwrap();

    //получать сообщения от потоков
    let (tx, rx) = mpsc::channel();

    let database = Arc::new(database);
    let data_custom = Arc::new(custom_digit);

    for _i in 0..num_cores {
        let clone_db = database.clone();
        let clone_dc = data_custom.clone();
        let tx = tx.clone();
        task::spawn_blocking(move || {
            process(&clone_db, bench, pazl,
                    &clone_dc, enum_start, tx, enum_end, enum_all, start_enum, end_enum, step, rnd_step);
        });
    }

    //отображает инфу в однy строку(обновляемую)
    let mut stdout = stdout();
    for received in rx {
        let list: Vec<&str> = received.split(",").collect();
        let mut speed = list[0].to_string().parse::<u64>().unwrap();
        speed = speed * num_cores as u64;
        print!("{}\rSPEED:{}/s|STEP:{}|{}", BACKSPACE, speed, list[2].to_string(), list[1].to_string());
        stdout.flush().unwrap();
    }
}

fn process(file_content: &Arc<HashSet<String>>, bench: bool, range: usize, custom: &Arc<String>, enum_start: usize, tx: Sender<String>,
           enum_end: usize, enum_all: u8, mut start_enum: u128, mut end_enum: u128, mut step: u128, rnd_step: bool) {
    let mut start = Instant::now();
    let mut speed: u32 = 0;
    let s = Secp256k1::new();
    let sk_def = SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000001460").unwrap();
    let enumall = string_to_bool(enum_all.to_string());

    //Заполняем сначала нужным количеством нулей
    let zero = start_zero(range);
    let mut rng = rand::thread_rng();

    //Известные
    let data_custom: Vec<&str> = custom.split(",").collect();
    //для скорости посмотрим есть ли они вообще
    let mut data_custom_run = false;
    for i in 0..range {
        if data_custom[i] != "*" {
            data_custom_run = true;
        }
    }

    //enum_start - сколько чисел слева переберать
    //enum_end - сколько чисел справа перебирать

    //start_enum - начальное значение пребора для 17  = 20000000000000000
    //end_enum - конец перебора, по умолчанию get_hex вернёт количествао F по enum_start

    //end_hex - конец перебора справа
    let end_hex = get_hex(enum_end);

    //если указана длинна 17 и начало не указано
    start_enum = if range == 17 && start_enum == 0 {
        get_hex_start17(enum_start)
    } else {
        if enum_start == 0 {
            0
        } else {
            start_enum
        }
    };

    end_enum = if end_enum>0{end_enum}else { get_hex(enum_start)};


    loop {
        //получаем рандомную строку нужной длиннны и устанавливаем пользовательские
        let randhex = if data_custom_run {
            let mut randr_str_and_user = "".to_string();
            for i in 0..range - (enum_start + enum_end) {
                if data_custom[i] != "*" {
                    randr_str_and_user.push_str(data_custom[i]);
                } else {
                    randr_str_and_user.push_str(HEX[rng.gen_range(0..=15)])
                }
            }
            randr_str_and_user
        } else {
            let mut randr_str_and_user = "".to_string();
            for _i in 0..range - (enum_start + enum_end) {
                randr_str_and_user.push_str(HEX[rng.gen_range(0..=15)])
            }
            randr_str_and_user
        };

        //если включен рандомный шаг
        if rnd_step {
            step = rng.gen_range(1..get_hex_rand_step(enum_start));
        }


        for end_h in 0..=end_hex {
            for start_h in (start_enum..=end_enum).step_by(step as usize) {
                //получаем готовый хекс пока без нулей
                let st = if end_enum == 0 { "".to_string() } else { format!("{:0enum_start$X}", start_h) };
                let en = if end_hex == 0 { "".to_string() } else { format!("{:0enum_end$X}", end_h) };

                let enum_hex_and_rand = format!("{st}{randhex}{en}");

                //если включен режим по очереди перебирая каждую рандомную
                if enumall {
                    for i in enum_start..range - enum_end {
                        for j in 0..=15 {
                            let mut st = enum_hex_and_rand.clone();
                            let mut rnd_str = start_zero(range);
                            st.replace_range(i..i + 1, HEX[j]);
                            rnd_str.push_str(&st);

                            let address = create_and_find(&rnd_str, file_content, &s, sk_def);
                            if bench {
                                println!("[{st}][{}][{address}]", hex_to_wif_compressed(hex::decode(&rnd_str).unwrap()));
                            } else {
                                speed = speed + 1;
                                if start.elapsed() >= Duration::from_secs(1) {
                                    tx.send(format!("{speed},{st},{step}").to_string()).unwrap();
                                    start = Instant::now();
                                    speed = 0;
                                }
                            }
                        }
                    }
                    //иначе напрямую
                } else {
                    let hex_string = format!("{zero}{enum_hex_and_rand}");

                    let address = create_and_find(&hex_string, file_content, &s, sk_def);
                    if bench {
                        println!("[{enum_hex_and_rand}][{}][{address}]", hex_to_wif_compressed(hex::decode(&hex_string).unwrap()));
                    } else {
                        speed = speed + 1;
                        if start.elapsed() >= Duration::from_secs(1) {
                            tx.send(format!("{speed},{enum_hex_and_rand},{step}").to_string()).unwrap();
                            start = Instant::now();
                            speed = 0;
                        }
                    }
                }
            }
        }
    }
}

fn get_hex_start17(range: usize) -> u128 {
    let hex = match range {
        1 => 0x2,
        2 => 0x20,
        3 => 0x200,
        4 => 0x2000,
        5 => 0x20000,
        6 => 0x200000,
        7 => 0x2000000,
        8 => 0x20000000,
        9 => 0x200000000,
        10 => 0x2000000000,
        11 => 0x20000000000,
        12 => 0x200000000000,
        13 => 0x2000000000000,
        14 => 0x20000000000000,
        15 => 0x200000000000000,
        16 => 0x2000000000000000,
        17 => 0x20000000000000000,
        _ => { 0x0 }
    };
    hex
}

fn get_hex(range: usize) -> u128 {
    let hex = match range {
        1 => 0xF,
        2 => 0xFF,
        3 => 0xFFF,
        4 => 0xFFFF,
        5 => 0xFFFFF,
        6 => 0xFFFFFF,
        7 => 0xFFFFFFF,
        8 => 0xFFFFFFFF,
        9 => 0xFFFFFFFFF,
        10 => 0xFFFFFFFFFF,
        11 => 0xFFFFFFFFFFF,
        12 => 0xFFFFFFFFFFFF,
        13 => 0xFFFFFFFFFFFFF,
        14 => 0xFFFFFFFFFFFFFF,
        15 => 0xFFFFFFFFFFFFFFF,
        16 => 0xFFFFFFFFFFFFFFFF,
        17 => 0xFFFFFFFFFFFFFFFFF,
        18 => 0xFFFFFFFFFFFFFFFFFF,
        19 => 0xFFFFFFFFFFFFFFFFFFF,
        20 => 0xFFFFFFFFFFFFFFFFFFFF,
        21 => 0xFFFFFFFFFFFFFFFFFFFFF,
        22 => 0xFFFFFFFFFFFFFFFFFFFFFF,
        23 => 0xFFFFFFFFFFFFFFFFFFFFFFF,
        24 => 0xFFFFFFFFFFFFFFFFFFFFFFFF,
        25 => 0xFFFFFFFFFFFFFFFFFFFFFFFFF,
        26 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFF,
        27 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        28 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        29 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        30 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        31 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        32 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        _ => { 0x0 }
    };
    hex
}

fn get_hex_rand_step(range: usize) -> u128 {
    let hex = match range {
        1 => 0x1,
        2 => 0x1,
        3 => 0x1,
        4 => 0x1,
        5 => 0x7,
        6 => 0xF,
        7 => 0xF,
        8 => 0xFF,
        9 => 0xFFF,
        10 => 0xFFFF,
        11 => 0xFFFFF,
        12 => 0xFFFFFF,
        13 => 0xFFFFFFF,
        14 => 0xFFFFFFFF,
        15 => 0xFFFFFFFFF,
        16 => 0xFFFFFFFFFF,
        17 => 0xFFFFFFFFFFF,
        18 => 0xFFFFFFFFFFFF,
        19 => 0xFFFFFFFFFFFFF,
        20 => 0xFFFFFFFFFFFFFF,
        21 => 0xFFFFFFFFFFFFFFF,
        22 => 0xFFFFFFFFFFFFFFFF,
        23 => 0xFFFFFFFFFFFFFFFFF,
        24 => 0xFFFFFFFFFFFFFFFFFF,
        25 => 0xFFFFFFFFFFFFFFFFFFF,
        26 => 0xFFFFFFFFFFFFFFFFFFFF,
        27 => 0xFFFFFFFFFFFFFFFFFFFFF,
        28 => 0xFFFFFFFFFFFFFFFFFFFFFF,
        29 => 0xFFFFFFFFFFFFFFFFFFFFFFF,
        30 => 0xFFFFFFFFFFFFFFFFFFFFFFFF,
        31 => 0xFFFFFFFFFFFFFFFFFFFFFFFFF,
        32 => 0xFFFFFFFFFFFFFFFFFFFFFFFFFF,
        _ => { 0x1 }
    };
    hex
}

fn create_and_find(hex: &String, file_content: &Arc<HashSet<String>>, s: &Secp256k1<All>, sk_def: SecretKey) -> String {
    let sk = SecretKey::from_str(&hex).unwrap_or(sk_def);
    let public_key_c = PublicKey::from_secret_key(&s, &sk);

    let address = get_legacy(&public_key_c.serialize());

    if file_content.contains(&address) {
        let private_key_c = hex_to_wif_compressed(hex::decode(&hex).expect(hex));
        print_and_save(&hex, &private_key_c, &address);
    }
    address
}

//legasy-----------------------------------------------------------------------
pub fn get_legacy(public_key: &[u8; 33]) -> String {
    let hash160 = hash160(&public_key.as_ref());
    let mut v = [0; 25];
    v[0] = 0x00;
    v[1..=20].copy_from_slice(&hash160.0);
    let checksum = sha256d(&v[0..=20]).0;
    v[21..=24].copy_from_slice(&checksum[0..=3]);
    v.to_base58()
}

//------------------------------------------------------------------------------------
fn hex_to_wif_compressed(raw_hex: Vec<u8>) -> String {
    let mut v = [0; 38];
    v[0] = 0x80;
    v[1..=32].copy_from_slice(&raw_hex.as_ref());
    v[33] = 0x01;
    let checksum = sha256d(&v[0..=33]).0;
    v[34..=37].copy_from_slice(&checksum[0..=3]);
    v.to_base58()
}

fn start_zero(p: usize) -> String {
    let r = match p {
        1 => "000000000000000000000000000000000000000000000000000000000000000".to_string(),
        2 => "00000000000000000000000000000000000000000000000000000000000000".to_string(),
        3 => "0000000000000000000000000000000000000000000000000000000000000".to_string(),
        4 => "000000000000000000000000000000000000000000000000000000000000".to_string(),
        5 => "00000000000000000000000000000000000000000000000000000000000".to_string(),
        6 => "0000000000000000000000000000000000000000000000000000000000".to_string(),
        7 => "000000000000000000000000000000000000000000000000000000000".to_string(),
        8 => "00000000000000000000000000000000000000000000000000000000".to_string(),
        9 => "0000000000000000000000000000000000000000000000000000000".to_string(),
        10 => "000000000000000000000000000000000000000000000000000000".to_string(),
        11 => "00000000000000000000000000000000000000000000000000000".to_string(),
        12 => "0000000000000000000000000000000000000000000000000000".to_string(),
        13 => "000000000000000000000000000000000000000000000000000".to_string(),
        14 => "00000000000000000000000000000000000000000000000000".to_string(),
        15 => "0000000000000000000000000000000000000000000000000".to_string(),
        16 => "000000000000000000000000000000000000000000000000".to_string(),
        17 => "00000000000000000000000000000000000000000000000".to_string(),
        18 => "0000000000000000000000000000000000000000000000".to_string(),
        19 => "000000000000000000000000000000000000000000000".to_string(),
        20 => "00000000000000000000000000000000000000000000".to_string(),
        21 => "0000000000000000000000000000000000000000000".to_string(),
        22 => "000000000000000000000000000000000000000000".to_string(),
        23 => "00000000000000000000000000000000000000000".to_string(),
        24 => "0000000000000000000000000000000000000000".to_string(),
        25 => "000000000000000000000000000000000000000".to_string(),
        26 => "00000000000000000000000000000000000000".to_string(),
        27 => "0000000000000000000000000000000000000".to_string(),
        28 => "000000000000000000000000000000000000".to_string(),
        29 => "00000000000000000000000000000000000".to_string(),
        30 => "0000000000000000000000000000000000".to_string(),
        31 => "000000000000000000000000000000000".to_string(),
        32 => "00000000000000000000000000000000".to_string(),
        33 => "0000000000000000000000000000000".to_string(),
        34 => "000000000000000000000000000000".to_string(),
        35 => "00000000000000000000000000000".to_string(),
        36 => "0000000000000000000000000000".to_string(),
        37 => "000000000000000000000000000".to_string(),
        38 => "00000000000000000000000000".to_string(),
        39 => "0000000000000000000000000".to_string(),
        40 => "000000000000000000000000".to_string(),
        41 => "00000000000000000000000".to_string(),
        42 => "0000000000000000000000".to_string(),
        43 => "000000000000000000000".to_string(),
        44 => "00000000000000000000".to_string(),
        45 => "0000000000000000000".to_string(),
        46 => "000000000000000000".to_string(),
        47 => "00000000000000000".to_string(),
        48 => "0000000000000000".to_string(),
        49 => "000000000000000".to_string(),
        50 => "00000000000000".to_string(),
        51 => "0000000000000".to_string(),
        52 => "000000000000".to_string(),
        53 => "00000000000".to_string(),
        54 => "0000000000".to_string(),
        55 => "000000000".to_string(),
        56 => "00000000".to_string(),
        57 => "0000000".to_string(),
        58 => "000000".to_string(),
        59 => "00000".to_string(),
        60 => "0000".to_string(),
        61 => "000".to_string(),
        62 => "00".to_string(),
        63 => "0".to_string(),
        64 => "".to_string(),
        _ => { "".to_string() }
    };
    r
}

fn print_and_save(hex: &str, key: &String, addres: &String) {
    println!("\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!("!!!!!!!!!!!!!!!!!!!!!!FOUND!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    println!("HEX:{}", hex);
    println!("PRIVATE KEY:{}", key);
    println!("ADDRESS:{}", addres);
    let s = format!("HEX:{}\nPRIVATE KEY: {}\nADDRESS {}\n", hex, key, addres);
    add_v_file("FOUND_PAZL.txt", s);
    println!("FOUND_PAZL.txt");
    println!("\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

fn add_v_file(name: &str, data: String) {
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(name)
        .expect("cannot open file")
        .write(data.as_bytes())
        .expect("write failed");
}

fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}
