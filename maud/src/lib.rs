//! A macro for writing HTML templates.
//!
//! This documentation only describes the runtime API. For a general
//! guide, check out the [book] instead.
//!
//! [book]: http://lfairy.gitbooks.io/maud/content/

use std::fmt;
use std::io;
use std::io::Read;

/// Escapes an HTML value.
pub fn escape(s: &str) -> Result<String, io::Error> {
    let mut buf = String::new();
    try!( rt::Escaper::new( s.as_bytes() ).read_to_string(&mut buf) );
    Ok(buf)
}

/// A block of HTML markup, as returned by the `html!` macro.
///
/// Use `.to_string()` to convert it to a `String`, or `.render()` to
/// write it directly to a handle.
pub struct Markup<F> {
    callback: F,
}

impl<F> Markup<F> where F: Fn(&mut fmt::Write) -> fmt::Result {
    /// Renders the markup to a `std::io::Write`.
    pub fn render<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        struct Adaptor<'a, W: ?Sized + 'a> {
            inner: &'a mut W,
            error: io::Result<()>,
        }

        impl<'a, W: ?Sized + io::Write> fmt::Write for Adaptor<'a, W> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    },
                }
            }
        }

        let mut output = Adaptor { inner: w, error: Ok(()) };
        match self.render_fmt(&mut output) {
            Ok(()) => Ok(()),
            Err(_) => output.error,
        }
    }

    /// Renders the markup to a `std::fmt::Write`.
    pub fn render_fmt(&self, w: &mut fmt::Write) -> fmt::Result {
        (self.callback)(w)
    }
}

impl<F> ToString for Markup<F> where F: Fn(&mut fmt::Write) -> fmt::Result {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        self.render_fmt(&mut buf).unwrap();
        buf
    }
}

/// Internal functions used by the `maud_macros` package. You should
/// never need to call these directly.
#[doc(hidden)]
pub mod rt {
    use std::fmt;
    use std::io::{Read, Error};
    use super::Markup;

    #[inline]
    pub fn make_markup<F>(f: F) -> Markup<F> where
        F: Fn(&mut fmt::Write) -> fmt::Result
    {
        Markup { callback: f }
    }

    pub struct Escaper<T: Read> {
        inner: T,
        buffer: [u8; 5],
        buffered: usize,
    }

    impl<T:Read> Escaper<T> {
        pub fn new(inner: T) -> Escaper<T> {
            Escaper {
                inner: inner,
                buffer: [0; 5],
                buffered: 0,
            }
        }
        
        // Attemps to write multiple character data to buf, otherwise stores in our buffer
        // Assumes that buffer is empty
        fn try_read(&mut self, mut data: &[u8], index: usize,  buf: &mut[u8]) -> Result<usize, Error> {
            debug_assert!( self.buffered == 0 );
            
            let wrote = try!( data.read( &mut buf[..index] ) );
            if wrote == data.len() { 
                return Ok(wrote); 
            }
            else {
                try!( (&self.buffer as &[u8]).read(&mut buf[..index + wrote]) );
                return Ok(wrote);
            }
        }
    }

    impl<T:Read> Read for Escaper<T> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            let mut local_buf = [0];
            let mut index = 0;

            if self.buffered != 0 {
                index += try!( (&self.buffer[..self.buffered] as &[u8]).read(buf) );
                self.buffered -= index;
            }
            
            if buf.len() == index { return Ok(index) };
            debug_assert!( self.buffered == 0 );
 
            while 0 != try!( self.inner.read(&mut local_buf) ) {
                index += match local_buf[0] {
                    b'&' => try!( self.try_read("&amp;".as_bytes(), index, buf) ),  
                    b'<' => try!( self.try_read("&lt;".as_bytes(), index, buf) ),
                    b'>' => try!( self.try_read("&gt;".as_bytes(), index, buf) ),
                    b'"' => try!( self.try_read("&quot;".as_bytes(), index, buf) ),
                    b'\'' => try!( self.try_read("&#39;".as_bytes(), index, buf) ),
                    other => { buf[index] = other; 1 }
                };
                
                if buf.len() == index { return Ok(index) };
            }

            return Ok(index);
        }
    }
}
