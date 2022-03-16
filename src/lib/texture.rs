use bevy::prelude::*;

pub fn get_size(
    opt_handle: Option<&Handle<Image>>,
    images: &Res<Assets<Image>>,
) -> Vec2 {
    if let Some(image) = opt_handle.map(|handle| images.get(handle)).flatten() {
        Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        )
    } else {
        Vec2::new(1., 1.)
    }
}
