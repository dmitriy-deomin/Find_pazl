mod data;

extern crate num_cpus;

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
use secp256k1::{All, PublicKey, Secp256k1};
use secp256k1::SecretKey;

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
    let enum_all: u8 = first_word(&conf[5].to_string()).to_string().parse::<u8>().unwrap();
    //---------------------------------------------------------------------

    //если указана длинна пазла больше звёздочек , дорисуем звёздочек
    let cash = custom_digit.clone();
    let cd: Vec<&str> = cash.split(",").collect();
    if cd.len() <= pazl {
        for i in 0..pazl {
            if cd.get(i as usize).is_none() {
                custom_digit.push_str(&*",*".to_string());
            }
        }
    }

    // Инфо блок
    // ---------------------------------------------------------------------
    println!("===============================");
    println!("FIND PAZL 66-160(17-40) v2.0.4");
    println!("===============================");

    println!("conf load:\n\
    -CPU CORE:{num_cores}/{count_cpu}\n\
    -HEX_END:{pazl}\n\
    -CUSTOM_DIGIT:\n{:?}\n\
    -ENUMERATION_START:{enum_start}\n\
    -ENUMERATION_END:{enum_end}\n\
    -ENUMERATION STEP 1 ALL:{}", custom_digit, string_to_bool(enum_all.clone().to_string()));
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

    println!("\nADRESS LOAD:{:?}", database.len());

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

    //добавляем варианты последовательного перебора если указаны
    let mut list_enum_start = vec![];
    let mut list_enum_end = vec![];

    //Заполняем списки для старта и конца
    let list_custom: Vec<&str> = custom_digit.split(",").collect();
    for i in 0..enum_start {
        if i == 0 {
            list_enum_start = get_enumerate_list(0, list_enum_start, list_custom[i as usize].to_string());
        } else {
            list_enum_start = get_enumerate_list(0, list_enum_start, list_custom[i as usize].to_string());
        }
    }
    for i in 0..enum_end {
        list_enum_end = get_enumerate_list(0, list_enum_end, list_custom[(pazl as usize - enum_end as usize) + i as usize].to_string());
    }

    let ss = list_enum_start.len();
    let se = list_enum_end.len();
    println!("ENUMERAT START:{ss}/1 RAND");
    println!("ENUMERAT END:{se}/1 RAND\n");

    //получать сообщения от потоков
    let (tx, rx) = mpsc::channel();

    let list_enum = Arc::new(list_enum_start);
    let list_enum_e = Arc::new(list_enum_end);
    let database = Arc::new(database);
    let data_custom = Arc::new(custom_digit);

    for _i in 0..num_cores {
        let clone_db = database.clone();
        let clone_dc = data_custom.clone();
        let clone_lu = list_enum.clone();
        let clone_lu_end = list_enum_e.clone();
        let tx = tx.clone();
        task::spawn_blocking(move || {
            process(&clone_db, bench, pazl,
                    &clone_dc, enum_start, tx, &clone_lu, enum_end, &clone_lu_end, enum_all);
        });
    }

    let alll = if enum_all == 1 { (pazl - enum_start) * 16 } else { 1 };
    //отображает инфу в однy строку(обновляемую)
    let mut stdout = stdout();
    for received in rx {
        let list: Vec<&str> = received.split(",").collect();
        let mut speed = list[0].to_string().parse::<u64>().unwrap();
        speed = speed * num_cores as u64;
        let rand_min = ((se + ss + 1) as f64 * alll as f64) / speed as f64;
        print!("{}\rSPEED:{}/s RAND:{} {}", BACKSPACE, speed, speed_to_time(rand_min), list[1].to_string());
        stdout.flush().unwrap();
    }
}

fn speed_to_time(s: f64) -> String {
    let r = if s < 60.0 {
        format!("{:.4}/s", s)
    } else if s < 3600.0 {
        format!("{:.4}/m", s / 60.0)
    } else {
        format!("{:.4}/h", (s / 60.0) / 60.0)
    };
    r
}

fn process(file_content: &Arc<HashSet<String>>, bench: bool, range: usize, custom: &Arc<String>, enum_start: usize, tx: Sender<String>,
           list_enum: &Arc<Vec<String>>, enum_end: usize, list_enum_end: &Arc<Vec<String>>, enum_all: u8) {
    let mut start = Instant::now();
    let mut speed: u32 = 0;

    let mut rng = rand::thread_rng();
    let s = Secp256k1::new();

    let sk_def = SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000001460").unwrap();

    let enumall = string_to_bool(enum_all.to_string());


    //Известные
    let data_custom: Vec<&str> = custom.split(",").collect();

    loop {
        //создаём случайные остальные числа если нет прямо указаных
        let mut hex_rand = "".to_string();
        for i in 0..range - (enum_start + enum_end) {
            if data_custom[i + enum_start] == "*" {
                hex_rand.push_str(&HEX[rng.gen_range(0..16)].to_string());
            } else {
                hex_rand.push_str(&data_custom[i + enum_start].to_string());
            }
        }

        //дописываем их к заготовке в начале
        let mut list_hex = vec![];
        if list_enum.len() > 0 {
            for i in list_enum.iter() {
                let hex = format!("{}{}", i, hex_rand.clone());
                list_hex.push(hex);
            }
        }

        //дописываем их к заготовке в конце
        if list_enum_end.len() > 0 {
            if list_hex.len() > 0 {
                let mut list_hex_cash = vec![];
                for y in list_hex.iter() {
                    for i in list_enum_end.iter() {
                        let hex = format!("{}{}", y, i);
                        list_hex_cash.push(hex);
                    }
                }
                list_hex = list_hex_cash.clone();
            } else {
                for i in list_enum_end.iter() {
                    let hex = format!("{}{}", hex_rand.clone(), i);
                    list_hex.push(hex);
                }
            }
        }

        //если нечего нет добавим как есть
        if list_enum.len() == 0 && list_enum_end.len() == 0 {
            list_hex.push(hex_rand);
        }

        for end in list_hex.iter() {

            //Заполняем сначала нужным количеством нулей
            let mut rnd_str = start_zero(range);

            if enumall{
                //по очереди перебирая каждую
                for i in enum_start..range-enum_end {
                    for j in 0..=15 {
                        let mut st = end.clone();
                        rnd_str = start_zero(range);

                        st.replace_range(i..i + 1, HEX[j]);

                        rnd_str.push_str(&st);

                        let address = create_and_find(&rnd_str, file_content, &s, sk_def);

                        if bench {
                            println!("[{st}][{}][{address}]",hex_to_wif_compressed(hex::decode(&rnd_str).unwrap()));
                        } else {
                            speed = speed + 1;
                            if start.elapsed() >= Duration::from_secs(1) {
                                tx.send(format!("{speed},[{st}]", ).to_string()).unwrap();
                                start = Instant::now();
                                speed = 0;
                            }
                        }
                    }
                }
            } else {
                //Добавляем остальные
                rnd_str.push_str(&end);

                let address = create_and_find(&rnd_str, file_content, &s, sk_def);

                if bench {
                    println!("[{end}][{}][{address}]",hex_to_wif_compressed(hex::decode(&rnd_str).unwrap()));
                } else {
                    speed = speed + 1;
                    if start.elapsed() >= Duration::from_secs(1) {
                        tx.send(format!("{speed},[{end}]", ).to_string()).unwrap();
                        start = Instant::now();
                        speed = 0;
                    }
                }
            }
        }
    }
}

fn create_and_find(hex: &String, file_content: &Arc<HashSet<String>>, s: &Secp256k1<All>, sk_def: SecretKey) -> String {
    let sk = SecretKey::from_str(&hex).unwrap_or(sk_def);
    let public_key_c = PublicKey::from_secret_key(&s, &sk);
    let address = get_legacy(&public_key_c.serialize());

    if file_content.contains(&address) {
        let private_key_c = hex_to_wif_compressed(hex::decode(&hex).unwrap());
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

fn get_enumerate_list(start: usize, mut list_enum: Vec<String>, custom_digit: String) -> Vec<String> {

    //если передаваемый список пуст заполним его
    if list_enum.len() == 0 {
        for (i, a) in HEX.iter().enumerate() {
            if custom_digit != "*".to_string() {
                list_enum.push(custom_digit.to_string());
            } else {
                let s = if start >= i {
                    HEX[start]
                } else {
                    a
                };
                list_enum.push(s.to_string());
            }
        }
        list_enum
    } else {
        let mut ret = vec![];
        for a in list_enum.iter() {
            for b in HEX.iter() {
                if custom_digit != "*".to_string() {
                    let end = format!("{a}{}", custom_digit.to_string());
                    ret.push(end);
                } else {
                    let end = format!("{a}{b}");
                    ret.push(end);
                }
            }
        }
        ret
    }
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
        _ => { "00000000000000000000000".to_string() }
    };
    r
}

fn print_and_save(hex: &String, key: &String, addres: &String) {
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
