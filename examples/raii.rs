struct Resource {
    data: i32,
}

impl Resource {
    fn new() -> Self {
        Self { data: 1337 }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        println!("resource dropped");
    }
}

fn main() {
    let ex = microseh::try_seh(|| unsafe {
        let res = Resource::new();
        println!("data: {}", res.data);
        std::ptr::read_volatile::<i32>(0 as _);
    });

    // U.B. if an exception is thrown, the resource will not be dropped!
    // You could choose to move the resource out of the closure like so:
    // let res = Resource::new();
    // let ex = microseh::try_seh(|| unsafe {
    //     println!("data: {}", res.data);
    //     std::ptr::read_volatile::<i32>(0 as _);
    // });

    if let Err(ex) = ex {
        println!("{:?}: {}", ex.address(), ex);
    }
}
