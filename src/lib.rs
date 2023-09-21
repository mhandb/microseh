use std::ffi::c_void;

use exception::Exception;

mod code;
mod exception;

type HandledProc = unsafe extern "system" fn(*mut c_void);

extern "system" {
    #[link_name = "HandlerStub"]
    fn handler_stub(proc: HandledProc, closure: *mut c_void, exception: *mut Exception) -> bool;
}

unsafe extern "system" fn handled_proc<F>(closure: *mut c_void)
where
    F: FnMut(),
{
    // Closure may be equal to std::ptr::null_mut() if the compiler optimized it away.
    // This also means that if you have some code that is optimized away, any exception it
    // contained will not get thrown.
    if let Some(closure) = closure.cast::<F>().as_mut() {
        closure();
    }
}

/// Executes a closure or function within a SEH-handled context.
///
/// # Arguments
///
/// * `closure` - The closure or function to be executed within the SEH-handled context.
///
/// # Returns
///
/// * `Ok(())` - If the closure executed without throwing any exceptions.
/// * `Err(Exception)` - If an exception occurred during the execution of the closure.
pub fn try_seh<F>(mut closure: F) -> Result<(), Exception>
where
    F: FnMut(),
{
    let mut exception = Exception::empty();
    let closure = &mut closure as *mut _ as *mut c_void;

    unsafe {
        match handler_stub(handled_proc::<F>, closure, &mut exception) {
            false => Err(exception),
            true => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::code::ExceptionCode;

    use super::*;

    #[test]
    fn access_violation() {
        let ex = try_seh(|| unsafe {
            let ptr = std::ptr::null_mut::<i32>();
            let _ = std::ptr::read_volatile(ptr);
        });

        assert_eq!(ex.is_err(), true);
        assert_eq!(ex.unwrap_err().code(), ExceptionCode::AccessViolation);
    }

    #[test]
    fn all_good() {
        let ex = try_seh(|| {
            let _ = *Box::new(1337);
        });

        assert_eq!(ex.is_ok(), true);
    }
}
