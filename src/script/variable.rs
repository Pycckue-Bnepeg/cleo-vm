extern crate alloc;

#[derive(Clone, PartialEq)]
pub enum VariableKind {
    Integer,
    Float,
    String,
}

pub struct Variable {
    pub kind: VariableKind,
    ptr: *mut u8,
    pub id: usize,
}

pub struct VarInfo {
	kind: VariableKind,
	value: usize,
}

pub enum VariableType {
    Global(usize),
    Local(usize),
}

impl Variable {
    pub fn new<T: Sized>(kind: VariableKind, value: T, id: usize) -> Variable {
        unsafe {
            let var = Variable {
                kind: kind,
                ptr: alloc::heap::allocate(32, 32),
                id: id,
            };

            *(var.ptr as *mut T) = value;

            var
        }
    }

    pub fn from_raw(id: usize, raw: &VarInfo) -> Variable {
        Variable::new(raw.kind.clone(), raw.value, id)
    }

    pub fn into_raw(&self) -> VarInfo {
        VarInfo {
            kind: self.kind.clone(),
            value: self.get(),
        }
    }

    pub fn from(&mut self, other: &Variable) {
        self.kind = other.kind.clone();
        self.set(other.get::<u32>());
    }

    pub fn clone(&self) -> Variable {
        Variable::new(self.kind.clone(), self.get::<u32>(), self.id)
    }

    pub fn do_stuff<F, T: Sized>(&mut self, other: &Variable, f: F) where F: Fn(T, T) -> T {
        let a = self.get::<T>();
        let b = other.get::<T>();
        self.set(f(a, b));
    }

    pub fn get<T: Sized>(&self) -> T {
        unsafe {
            ::std::ptr::read(self.ptr as *const T)
        }
    }

    pub fn set<T: Sized>(&mut self, value: T) {
        unsafe {
            *(self.ptr as *mut T) = value;
        }
    }

    pub fn get_str(&self) -> String {
        unsafe {
            ::std::ffi::CString::from_raw(self.get()).into_string().unwrap()
        }
    }

    pub fn set_str(&mut self, val: String) {
        unsafe {
            let len = val.bytes().len();
            let ptr = alloc::heap::allocate(len + 1, 8);
            ::std::ptr::copy(val.as_ptr(), ptr, len);
            self.set(ptr as usize);
        }
    }

    pub fn change(&mut self, kind: VariableKind) {
        self.kind = kind;
    }

    pub fn eq_types(&self, other: &Variable) -> bool {
        self.kind == other.kind
    }
}

impl Drop for Variable {
    fn drop(&mut self) {
        unsafe {
            alloc::heap::deallocate(self.ptr, 32, 32);
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Variable) -> bool {
        self.eq_types(&other) && self.get::<u32>() == other.get::<u32>()
    }
}

impl ::std::fmt::Display for Variable {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self.kind {
            VariableKind::Integer => write!(f, "{}@: integer {}", self.id, self.get::<u32>()),
            VariableKind::Float => write!(f, "{}@: float {}", self.id, self.get::<f32>()),
            VariableKind::String => write!(f, "{}@: string \"{}\"", self.id, self.get_str()),
        }
    }
}