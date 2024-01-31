pub fn get_conf_text()->String{
    let t = format!("1 -CPU core 0/{} (0 - test mode)\n\
            17 -hex end(17-40). 66,67,68 pazl = 17 digit\n\
            *,*,*,*,*,*,*,*,*,*,*,*,*,*,*,*,* -custom digit(0123456789ABCDEF)\n\
            4 -ENUMERATION start digit(MAX 32)\n\
            0 -ENUMERATION end digit(MAX 32)\n\
            0 -ENUMERATION STEP 1 ALL(0/1)\n\
            ==========The alternative===========\n\
            0 -START ENUMERATION(66 pazl = 20000000000000000/0 of)\n\
            0 -STOP ENUMERATION(66 pazl = 3FFFFFFFFFFFFFFFF/0 of)\n\
            1 -STEP\n\
            0 -RAND STEP(0/1)\n\
            0,1,2,3,4,5,6,7,8,9,A,B,C,D,E,F -custom HEX(0,1,2,3,4,5,6,7,8,9,A,B,C,D,E,F) \n\n\
            ----------------------------------------------------------\n\
            info(no settings)\n\
            66 - 20000000000000000 to 3FFFFFFFFFFFFFFFF\n\
            67 - 40000000000000000 to 7ffffffffffffffff\n\
            68 - 80000000000000000 to fffffffffffffffff\n\
            found to be saved FOUND_PAZL.txt\n\
            donate:bc1qg89l3580w7zgqkc54kufgpdyk3ur88d772l9y0", num_cpus::get());
    t.to_string()
}