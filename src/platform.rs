//! Utilidades especificas del sistema operativo. minifb no ofrece una forma
//! multiplataforma de confinar el cursor dentro de la ventana, asi que en
//! Windows llamamos directamente a la API de Win32 (`ClipCursor`) via FFI;
//! en cualquier otro sistema operativo son funciones vacias (el mouse-look
//! sigue funcionando, solo que el cursor puede salirse de la ventana).

#[cfg(target_os = "windows")]
mod imp {
    use std::os::raw::c_void;

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetClientRect(hwnd: *mut c_void, rect: *mut Rect) -> i32;
        fn ClientToScreen(hwnd: *mut c_void, point: *mut Point) -> i32;
        fn ClipCursor(rect: *const Rect) -> i32;
        fn SetCursorPos(x: i32, y: i32) -> i32;
    }

    /// Restringe el cursor del sistema al area cliente de la ventana del
    /// juego (en coordenadas de pantalla). Se llama cada cuadro porque
    /// Windows libera el confinamiento cuando la ventana pierde el foco.
    pub fn confine_cursor(hwnd: *mut c_void) {
        unsafe {
            let mut rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
            if GetClientRect(hwnd, &mut rect) == 0 {
                return;
            }

            let mut top_left = Point { x: rect.left, y: rect.top };
            let mut bottom_right = Point { x: rect.right, y: rect.bottom };
            ClientToScreen(hwnd, &mut top_left);
            ClientToScreen(hwnd, &mut bottom_right);

            let clip = Rect {
                left: top_left.x,
                top: top_left.y,
                right: bottom_right.x,
                bottom: bottom_right.y,
            };
            ClipCursor(&clip);
        }
    }

    /// Libera el confinamiento del cursor; se llama al salir del juego.
    pub fn release_cursor() {
        unsafe {
            ClipCursor(std::ptr::null());
        }
    }

    /// Vuelve a poner el cursor en el centro de la ventana (en coordenadas
    /// de pantalla). Confinar el cursor no basta para un mouse-look
    /// continuo: tarde o temprano llega al borde de la ventana y deja de
    /// generar movimiento. Recentrarlo cada cuadro (y calcular la rotacion
    /// a partir de esa posicion central, no de la del cuadro anterior) es
    /// la tecnica estandar para lograr una rotacion sin limites.
    pub fn recenter_cursor(hwnd: *mut c_void, client_width: i32, client_height: i32) {
        unsafe {
            let mut center = Point { x: client_width / 2, y: client_height / 2 };
            if ClientToScreen(hwnd, &mut center) == 0 {
                return;
            }
            SetCursorPos(center.x, center.y);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use std::os::raw::c_void;

    pub fn confine_cursor(_hwnd: *mut c_void) {}
    pub fn release_cursor() {}
    pub fn recenter_cursor(_hwnd: *mut c_void, _client_width: i32, _client_height: i32) {}
}

pub use imp::{confine_cursor, recenter_cursor, release_cursor};
