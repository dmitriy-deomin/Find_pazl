pub fn get_conf_text()->String{
    let t = format!("1 -CPU core 0/{} (0 - test mode)\n\
            17 -hex end(17-40). 66,67,68 pazl = 17 digit\n\
            *,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,* -custom digit(0123456789ABCDEF)\n\
            4 -ENUMERATION start digit(MAX 32)\n\
            0 -ENUMERATION end digit(MAX 32)\n\
            0 -ENUMERATION STEP 1 ALL(0/1)\n\n\
            ----------------------------------------------------------\n\
            found to be saved FOUND_PAZL.txt\n\
            donate:bc1qg89l3580w7zgqkc54kufgpdyk3ur88d772l9y0", num_cpus::get());
    t.to_string()
}