use std::ffi::CString;
use std::{ mem, os::raw::c_void };
use glm::Scalar;
use itertools::Itertools;

// glm utils

#[allow(unused)]
/// Convert an array of Vec2 into an array of numbers
pub fn from_array_of_vec2<T: Scalar + Copy>(arr: Vec<glm::TVec2<T>>) -> Vec<T> {
    arr.iter()
    .map(|v| vec![v[0], v[1]])
    .flatten()
    .collect::<_>()
}
#[allow(unused)]
/// Convert an array of Vec3 into an array of numbers
pub fn from_array_of_vec3<T: Scalar + Copy>(arr: Vec<glm::TVec3<T>>) -> Vec<T> {
    arr.iter()
    .map(|v| vec![v[0], v[1], v[2]])
    .flatten()
    .collect::<_>()
}
#[allow(unused)]
/// Convert an array of Vec4 into an array of numbers
pub fn from_array_of_vec4<T: Scalar + Copy>(arr: Vec<glm::TVec4<T>>) -> Vec<T> {
    arr.iter()
        .map(|v| vec![v[0], v[1], v[2], v[3]])
        .flatten()
        .collect::<_>()
}
#[allow(unused)]
/// Convert an array of numbers representing 2-tuples to array of vec2
pub fn to_array_of_vec2<T: Scalar + Copy>(arr: Vec<T>) -> Vec<glm::TVec2<T>> {
    arr.iter()
    .chunks(2)
    .into_iter()
    .map(|mut step| glm::vec2(
        *step.next().unwrap(), 
        *step.next().unwrap()
    ))
    .collect::<_>()
}
#[allow(unused)]
/// Convert an array of numbers representing 3-tuples to array of vec3
pub fn to_array_of_vec3<T: Scalar + Copy>(arr: Vec<T>) -> Vec<glm::TVec3<T>> {
    arr.iter()
    .chunks(3)
    .into_iter()
    .map(|mut step| glm::vec3(
        *step.next().unwrap(), 
        *step.next().unwrap(),
        *step.next().unwrap(),
    ))
    .collect::<_>()
}
#[allow(unused)]
/// Convert an array of numbers representing 4-tuples to array of vec4
pub fn to_array_of_vec4<T: Scalar + Copy>(arr: Vec<T>) -> Vec<glm::TVec4<T>> {
    arr.iter()
        .chunks(4)
        .into_iter()
        .map(|mut step| glm::vec4(
            *step.next().unwrap(), 
            *step.next().unwrap(),
            *step.next().unwrap(),
            *step.next().unwrap(),
        ))
        .collect::<_>()
}

#[allow(unused)]
// Get an offset in bytes for n units of type T
pub fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

pub fn vec4_f32_to_f64(v: &glm::TVec4<f32>) -> glm::TVec4<f64> {
    glm::vec4(v.x as _, v.y as _, v.z as _, v.w as _)
}
pub fn vec4_f64_to_f632(v: &glm::TVec4<f64>) -> glm::TVec4<f32> {
    glm::vec4(v.x as _, v.y as _, v.z as _, v.w as _)
}
pub fn vec3_f32_to_f64(v: &glm::TVec3<f32>) -> glm::TVec3<f64> {
    glm::vec3(v.x as _, v.y as _, v.z as _)
}
pub fn vec3_f64_to_f632(v: &glm::TVec3<f64>) -> glm::TVec3<f32> {
    glm::vec3(v.x as _, v.y as _, v.z as _)
}
pub fn vec2_f32_to_f64(v: &glm::TVec2<f32>) -> glm::TVec2<f64> {
    glm::vec2(v.x as _, v.y as _)
}
pub fn vec2_f64_to_f632(v: &glm::TVec2<f64>) -> glm::TVec2<f32> {
    glm::vec2(v.x as _, v.y as _)
}

pub unsafe fn get_gl_string(name: gl::types::GLenum) -> String {
    std::ffi::CStr::from_ptr(gl::GetString(name) as *mut i8).to_string_lossy().to_string()
}

// Debug callback to panic upon enountering any OpenGL error
pub extern "system" fn debug_callback(
    source: u32, e_type: u32, id: u32,
    severity: u32, _length: i32,
    msg: *const i8, _data: *mut std::ffi::c_void
) {
    if e_type != gl::DEBUG_TYPE_ERROR { return }
    if severity == gl::DEBUG_SEVERITY_HIGH ||
       severity == gl::DEBUG_SEVERITY_MEDIUM ||
       severity == gl::DEBUG_SEVERITY_LOW
    {
        let severity_string = match severity {
            gl::DEBUG_SEVERITY_HIGH => "high",
            gl::DEBUG_SEVERITY_MEDIUM => "medium",
            gl::DEBUG_SEVERITY_LOW => "low",
            _ => "unknown",
        };
        unsafe {
            let string = CString::from_raw(msg as *mut i8);
            let error_message = String::from_utf8_lossy(string.as_bytes()).to_string();
            panic!("{}: Error of severity {} raised from {}: {}\n",
                id, severity_string, source, error_message);
        }
    }
}

