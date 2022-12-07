pub fn test_target(buf: &[u8]) -> Result<Vec<(u32,u32)>, usize>{
    let mut res: Vec<(u32, u32)> = vec![];

    res.push((0,0));

    if buf.len() == 11 {
        if buf[0] as char == 'f' {
            //dprintln!("f");
            res.push((0,1));

            if buf[1] as char == 'u' {
                //dprintln!("u");
                res.push((0,2));

                if buf[2] as char == 'z' {
                    //dprintln!("z");
                    res.push((0,3));

                    if buf[3] as char == 'z' {
                        //dprintln!("z");
                        res.push((0,4));

                        if buf[4] as char == 'i' {
                            //dprintln!("i");
                            res.push((0,5));

                            if buf[5] as char == 'n' {
                                //dprintln!("n");
                                res.push((0,6));

                                if buf[6] as char == 'g' {
                                    //dprintln!("g");
                                    res.push((0,7));

                                    panic!("gg {:?}", buf);
                                }


                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(res);
}