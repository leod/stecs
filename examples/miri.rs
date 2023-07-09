use memoffset::offset_of;

#[repr(C)]
struct Entity {
    x: u32,
    y: f32,
}

unsafe fn borrow_x<'a>(entity: *mut Entity) -> &'a u32 {
    let entity = entity as *mut Entity as *mut u8;
    let offset = offset_of!(Entity, x);

    let ptr = unsafe { entity.add(offset_of!(Entity, x)) };

    println!("x at {offset} -> {ptr:?}");

    let component = ptr as *mut u32;

    unsafe { &*component }
}

unsafe fn borrow_y<'a>(entity: *mut Entity) -> &'a mut f32 {
    let entity = entity as *mut Entity as *mut u8;
    let offset = offset_of!(Entity, y);

    let ptr = unsafe { entity.add(offset_of!(Entity, y)) };

    println!("y at {offset} -> {ptr:?}");

    let component = ptr as *mut f32;

    unsafe { &mut *component }
}

fn split(entity: &mut Entity) -> (&u32, &mut f32) {
    let ptr = entity as *mut Entity;

    let x = unsafe { borrow_x(ptr) };
    let y = unsafe { borrow_y(ptr) };

    (x, y)
}

fn main() {
    let mut entity = Entity { x: 17, y: 2099.3 };

    let (x, y) = split(&mut entity);

    println!("{} {}", *x, *y);

    *y = 2.0;
    *y += *x as f32;

    println!("{} {}", *x, *y);
}
