pub fn get_conf_text()->String{
    let t = format!(
           "1   -КОЛИЧЕСТВО ПОТОКОВ ПРОЦЕССОРА 1/{}\n\
            17  -ДЛИННА ПАЗЛА(17-40). 66,67,68 ПАЗЛ = 17 ЗНАКОВ\n\
            1a838b13505b26867 -НАЧАЛЬНОЕ ЗНАЧЕНИЕ(0123456789ABCDEF)(для теста 65 пазл 1a838b13505b26867)\n\
            7   -КОЛИЧЕСТВО СИМВОЛОВ ПОСЛЕДОВАТЕЛЬНОГО ПЕРЕБОРА СЛЕВА\n\
            0123456789ABCDEF -АЛФАВИТ(0123456789ABCDEF)\n\
            1   -ОТОБРАЖЕНИЕ СКОРОСТИ И ТЕКУЩЕГО ПОДБОРА(0-выкл, 1-включенно) \n\n
            Найденное сохраниться в  FOUND_PAZL.txt\n\
            ---Задонатить:------\n\
            (BTC)              bc1qg89l3580w7zgqkc54kufgpdyk3ur88d772l9y0\n\
            (KASPA)            kaspa:qpjmst279twpa48yyql3frxk8k2fsa6sh2pky7szv27s9ftq0wwssaffg58up\n\
            (TONCOIN telegram) UQD4ULmR2ddYigLQ82D-_MPbSXkIzasHA73JFg1-hY4l-Ft4", num_cpus::get());
    t.to_string()
}