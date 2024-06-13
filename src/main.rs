mod data;
mod color;
extern crate rand;
extern crate num_cpus;
use std::{collections::HashSet, fs::{OpenOptions, File}, time::{Instant, Duration}, io::{BufRead, BufReader, Write}, path::Path, io, thread};
use std::io::stdout;
use std::sync::{Arc, mpsc};
use rand::Rng;
use base58::{FromBase58, ToBase58};
use sha2::{Digest, Sha256};
use crate::color::{blue, cyan, green, magenta};
use sv::util::hash160;
use rustils::parse::boolean::string_to_bool;

#[cfg(not(windows))]
use rust_secp256k1::{PublicKey, Secp256k1, SecretKey};

#[cfg(windows)]
mod ice_library;

#[cfg(windows)]
use ice_library::IceLibrary;

const FILE_CONFIG: &str = "confPazl.txt";
const BACKSPACE: char = 8u8 as char;

#[tokio::main]
async fn main() {
    //для измерения скорости
    let mut start = Instant::now();
    let mut speed: u32 = 0;
    let one_sek = Duration::from_secs(1);
    let mut rng = rand::thread_rng();

    let conf = lines_from_file(&FILE_CONFIG).unwrap_or_else(|_| {
        add_v_file(&FILE_CONFIG, data::get_conf_text().to_string());
        lines_from_file(&FILE_CONFIG).expect("Failed to read config file")
    });

    let num_cores: i8 = first_word(&conf[0]).parse().expect("Failed to parse num_cores");
    let pazl: usize = first_word(&conf[1]).parse().expect("Failed to parse pazl");
    let custom_digit_start = first_word(&conf[2]).to_uppercase();
    let enum_start: usize = first_word(&conf[3]).parse().expect("Failed to parse enum_start");
    let alvabet = first_word(&conf[4]).to_string();
    let show_info = string_to_bool(first_word(&conf[5].to_string()).to_string());
    //---------------------------------------------------------------------

    // Инфо блок
    //---------------------------------------------------------------------------------------------------
    display_configuration_info(num_cores, pazl, &custom_digit_start, enum_start,&alvabet);
    //-------------------------------------------------------------------------------------------------

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
        let binding = address.from_base58().unwrap();
        let mut a=[0; 20];

        a.copy_from_slice(&binding.as_slice()[1..=20]);
        database.insert(a);
    }

    println!("{}{:?}", blue("АДРЕСОВ В БАЗЕ:"), green(database.len()));

    //главные каналы
    let (main_sender, main_receiver) = mpsc::channel();

    //будет храниться список запушеных потоков(каналов для связи)
    let mut channels = Vec::new();
    let database = Arc::new(database);


    let charset_chars: Vec<char> = alvabet.chars().collect();
    let charset_len = charset_chars.len();

    //состовляем начальную позицию
    let mut current_combination = vec![0; pazl];

    //заполняем страртовыми значениями
    for d in 0..pazl {
        let position = match custom_digit_start.chars().nth(d) {
            Some(ch) => {
                // Находим позицию символа в charset_chars
                charset_chars.iter().position(|&c| c == ch).unwrap_or_else(|| rng.gen_range(0..charset_len) )
            }
            None => { rng.gen_range(0..charset_len) }
        };
        current_combination[d] = position;
    }

    // создание потоков
    for ch in 0..num_cores as usize {
        let (sender, receiver) = mpsc::channel();
        let clone_db = database.clone();

        let main_sender = main_sender.clone();

        #[cfg(windows)]
            let ice_library = {
            let lib = IceLibrary::new();
            lib.init_secp256_lib();
            lib
        };

        //для всего остального
        #[cfg(not(windows))]
            let secp = Secp256k1::new();

        // Поток для выполнения задач,работает постоянно принимает сообщения и шлёт
        thread::spawn(move || {
            loop {
                let h:String = receiver.recv().unwrap();

                // Получаем публичный ключ для разных систем , адрюха не дружит с ice_library
                //------------------------------------------------------------------------
                #[cfg(windows)]
                    let pk_c={
                    ice_library.privatekey_to_publickey(&h.as_str())
                };

                #[cfg(not(windows))]
                    let pk_c= {
                    // Создаем секретный ключ из байт
                    let secret_key = SecretKey::from_slice(&h.as_str()).expect("32 bytes, within curve order");
                    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
                    public_key.serialize()
                };
                //----------------------------------------------------------------------------

                //получем из них хеш160
                let h160c = hash160(&pk_c[0..]).0;

                //проверка наличия в базе BTC compress
                if clone_db.contains(&h160c) {
                    let address = get_legacy(h160c, 0x00);
                    print_and_save(h,address);
                }

                //шлём поток
                main_sender.send(ch).unwrap();
            }
        });
        // Инициализация
        let ch = vec!['a'];
        sender.send(current_combination.iter().map(|&idx| ch[0]).collect()).unwrap();
        channels.push(sender);
    }
    //------------------------------------------------------------------------------

    //слушаем ответы потков и если есть шлём новую задачу
    for received in main_receiver {
        let ch = received;

        // следующая комбинация пароля если алфавит пустой будем по всем возможным перебирать
        let password_string:String = current_combination.iter().map(|&idx| charset_chars[idx]).collect();

        //измеряем скорость и шлём прогресс
        if show_info{
            speed += 1;
            if start.elapsed() >= one_sek {
                let mut stdout = stdout();
                print!("{}\r{}", BACKSPACE, green(format!("SPEED:{speed}/s|{password_string}")));
                stdout.flush().unwrap();
                start = Instant::now();
                speed = 0;
            }
        }

        // Отправляем новую в свободный канал
        channels[ch].send(password_string).unwrap();

        //будем переберать слева указаное количество
        let mut i = enum_start;
        while i > 0 {
            i -= 1;
            if current_combination[i] + 1 < charset_len {
                current_combination[i] += 1;
                break;
            } else {
                current_combination[i] = 0;
            }
        }

        //конец
        if current_combination[0] == 0 {
            for d in 0..pazl {
                let position = match custom_digit_start.chars().nth(d) {
                    Some(ch) => {
                        // Находим позицию символа в charset_chars
                        charset_chars.iter().position(|&c| c == ch).unwrap_or_else(|| rng.gen_range(0..charset_len) )
                    }
                    None => { rng.gen_range(0..charset_len) }
                };
                current_combination[d] = position;
            }
        }
    }
}
//------------------------------------------------------------------------------------

fn sha256d(data: &[u8]) -> Vec<u8> {
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(&first_hash);
    second_hash.to_vec()
}

fn start_zero(p: usize) -> String {
    if p >= 64 {
        return "".to_string();
    }
    // Создаем строку, состоящую из p нулей
    "0".repeat(64 - p)
}

fn print_and_save(hex: String, addres: String) {
    println!("{}", cyan("\n!!!!!!!!!!!!!!!!!!!!!!FOUND!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"));
    println!("{}{}", cyan("HEX:"), cyan(hex.clone()));
    println!("{}{}", cyan("ADDRESS:"), cyan(addres.clone()));
    let s = format!("HEX:{}\nADDRESS {}\n", hex, addres);
    add_v_file("FOUND_PAZL.txt", s);
    println!("{}", cyan("СОХРАНЕНО В FOUND_PAZL.txt"));
    println!("{}", cyan("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"));
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

pub fn get_legacy(hash160: [u8; 20], coin: u8) -> String {
    let mut v = Vec::with_capacity(23);
    v.push(coin);
    v.extend_from_slice(&hash160);
    let checksum = sha256d(&v);
    v.extend_from_slice(&checksum[0..4]);
    let b: &[u8] = v.as_ref();
    b.to_base58()
}

fn first_word(s: &String) -> &str {
    s.trim().split_whitespace().next().unwrap_or("")
}

fn display_configuration_info(num_cores: i8, pazl: usize, custom_digit: &str, enum_start: usize, custom_hex: &str) {
    println!("{}", blue("==============================="));
    println!("{} {}", blue("FIND PAZL 66-160(17-40)"),magenta(env!("CARGO_PKG_VERSION")));
    println!("{}", blue("==============================="));

    println!("{conf_load}\n\
    {cpu_core}{}{palka}{}\n\
    {end_hex}{}\n\
    {customdigit}{:?}\n\
    {enumstart}{}\n\
    {hhhh}", green(num_cores), blue(num_cpus::get()), green(pazl), green(custom_digit), green(enum_start),
             conf_load = blue("conf load:"), cpu_core = blue("КОЛИЧЕСТВО ПОТОКОВ:"), end_hex = blue("ДЛИННА ПАЗЛА:"),
             customdigit = blue("НАЧАЛО ПЕРЕБОРА"), enumstart = blue("КОЛИЧЕСТВО СИМВОЛОВ ПОСЛЕДОВАТЕЛЬНОГО ПЕРЕБОРА СЛЕВА:"),
             palka = blue("/"), hhhh = format!("{}{}", blue("АЛФАВИТ:"), green(custom_hex)));
}
