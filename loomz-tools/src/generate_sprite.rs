//! Generate optimized sprites from the data in `assets/dev/tiny_sword`
//! Call this script using `cargo run -p loomz-tools --release -- -c generate_sprites -f [optional_filters]`
use std::{fs::File, u32};
use crate::shared::{Rect, Size, Offset, size};

const SRC_ROOT: &str = "assets/dev/tiny_sword/";
const DST_ROOT: &str = "assets/dev/textures/";
const PADDING_PX: u32 = 2;

struct OutputInfo(png::OutputInfo);

impl OutputInfo {
    const fn default() -> Self { OutputInfo ( png::OutputInfo { width: 0, height: 0, line_size: 0, bit_depth: png::BitDepth::One, color_type: png::ColorType::Grayscale } ) }
}

impl std::ops::Deref for OutputInfo {
    type Target = png::OutputInfo;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::ops::DerefMut for OutputInfo {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Clone for OutputInfo {
    fn clone(&self) -> Self {
        OutputInfo ( png::OutputInfo { width: self.width, height: self.height, color_type: self.color_type, bit_depth: self.bit_depth, line_size: self.line_size } )
    }
}

#[derive(Clone)]
struct ActorAnimation {
    src_rect: Vec<Rect>,
    src_offset: Vec<Rect>,  // Left / Bottom offsets for centering the dst sprite
    dst_rect: Vec<Rect>,
    dst_sprite_offset: Rect,
    dst_sprite_size: Size,

    /// Top left corner of the first sprite in the destination image
    dst_sprite_start: Offset,
}

/// All the info required to read and process a "actor" sprites 
#[derive(Clone)]
struct Actor {
    file_name: String,
    src_path: &'static str,
    dst_path: String,
    dst_path_json: String,
    src_sprite_size: Size,
    src_image_info: OutputInfo,
    dst_image_info: OutputInfo,
    src_bytes: Vec<u8>,
    dst_bytes: Vec<u8>,
    animations: Vec<ActorAnimation>,
    animation_names: &'static [&'static str],
    loaded: bool,
}

#[derive(Default)]
struct AssetsState {
    actors: Vec<Actor>
}

//
// Loading files & templates
//

fn load_actors_source(actor: &mut Actor) {
    let path = format!("{SRC_ROOT}{}", actor.src_path);

    let decoder = png::Decoder::new(File::open(&path).unwrap());
    let mut reader = decoder.read_info().unwrap();

    actor.src_bytes = vec![0; reader.output_buffer_size()];
    actor.src_image_info.0 = reader.next_frame(&mut actor.src_bytes).unwrap();
    actor.loaded = true;
}

fn load_actors(filters: &[String], state: &mut AssetsState) {
    fn actor(src_path: &'static str, src_sprite_size: Size, animation_names: &'static [&'static str]) -> Actor {
        let file_name = ::std::path::Path::new(&src_path).file_stem().and_then(|n| n.to_str() ).unwrap();
        Actor {
            file_name: file_name.to_string(),
            src_path,
            dst_path: format!("{DST_ROOT}{file_name}.png"),
            dst_path_json: format!("{DST_ROOT}{file_name}.json"),
            src_sprite_size,
            src_image_info: OutputInfo::default(),
            dst_image_info: OutputInfo::default(),
            src_bytes: Vec::new(),
            dst_bytes: Vec::new(),
            animations: Vec::new(),
            animation_names,
            loaded: false,
        }
    }

    let mut actors = Vec::with_capacity(10);
    actors.push(actor("Factions/Knights/Troops/Pawn/Blue/Pawn_Blue.png", size(192, 192), &["idle", "walk", "hammer", "axe", "idle_hold", "idle_walk"]));
    actors.push(actor("Factions/Knights/Troops/Warrior/Blue/Warrior_Blue.png", size(192, 192), &["idle", "walk", "strike-horz-1", "strike-horz-2", "strike-bottom-1", "strike-bottom-2", "strike-top-1", "strike-top-2"]));
    
    for actor in actors.iter() {
        if !filters.is_empty() {
            if !filters.iter().any(|f| actor.src_path.matches(f).next().is_some() ) {
                continue;
            }
        }
        
        let mut actor = actor.clone();
        load_actors_source(&mut actor);
        state.actors.push(actor);
    }
}

//
// Reading actor sprites from source
//

fn scan_sprite<P: Copy+Default+PartialEq>(actor: &mut Actor, animation: &mut ActorAnimation, scan_x: &[usize; 3], scan_y: &[usize; 3]) {
    let pixels = actor.src_bytes.as_mut_ptr() as *mut P;
    let scanline_pixel = actor.src_image_info.line_size  / size_of::<P>();
    let zero = P::default();

    let mut rect = Rect { left: u32::MAX, top: u32::MAX, right: 0, bottom: 0 };

    for y in scan_y[0]..scan_y[1] {
        for x in scan_x[0]..scan_x[1] {
            let index = (y*scanline_pixel) + x;
            let pixel = unsafe { pixels.add(index).read() };
            if pixel != zero {
                rect.left = u32::min(rect.left, x as u32);
                rect.right = u32::max(rect.right, (x+1) as u32);
                rect.top = u32::min(rect.top, y as u32);
                rect.bottom = u32::max(rect.bottom, (y+1) as u32);
            }
        }
    }

    if rect.left == u32::MAX {
        return;
    }

    let [center_x, center_y] = [scan_x[2] as u32, scan_y[2] as u32];
    let offset = Rect {
        left: center_x - rect.left,
        top: center_y - rect.top,
        right: rect.right - center_x,
        bottom: rect.bottom - center_y,
    };

    animation.src_rect.push(rect);
    animation.src_offset.push(offset);
}

fn build_actor_animation<P: Copy+Default+PartialEq>(actor: &mut Actor, animation_index: usize) {
    let cell_width = actor.src_sprite_size.width as usize;
    let cell_height = actor.src_sprite_size.height as usize;

    let sprite_count = actor.src_image_info.width as usize / cell_width;

    let y1 = animation_index * cell_height;
    let y2 = y1 + cell_height;
    let y3 = y1 + ((y2 - y1) / 2);

    let mut animation = ActorAnimation {
        src_rect: Vec::with_capacity(sprite_count),
        src_offset: Vec::with_capacity(sprite_count),
        dst_rect: Vec::with_capacity(sprite_count),
        dst_sprite_offset: Rect::default(),
        dst_sprite_size: Size::default(),
        dst_sprite_start: Offset::default(),
    };

    for i in 0..sprite_count {
        let x1 = i*cell_width;
        let x2 = x1 + cell_width;
        let x3 = x1 + ((x2 - x1) / 2);
        scan_sprite::<P>(actor, &mut animation, &[x1, x2, x3], &[y1, y2, y3]);
    }

    actor.animations.push(animation);
}

fn find_actor_animation_sources<P: Copy+Default+PartialEq>(actor: &mut Actor) {
    let cell_height = actor.src_sprite_size.height;
    let animations_count = actor.src_image_info.height / cell_height;
    actor.animations = Vec::with_capacity(animations_count as usize);
    for i in 0..animations_count {
        build_actor_animation::<P>(actor, i as usize);
    }
}

//
// Generating output
//

fn prepare_actor_dst<P>(actor: &mut Actor) {
    let pixel_size = size_of::<P>();
    let mut dst_width = 0;
    let mut dst_height = 0;
    
    for animation in actor.animations.iter_mut() {
        let mut offsets_max = Rect::default();
        for offsets in animation.src_offset.iter() {
            offsets_max.left = u32::max(offsets_max.left, offsets.left);
            offsets_max.top = u32::max(offsets_max.top, offsets.top);
            offsets_max.right = u32::max(offsets_max.right, offsets.right);
            offsets_max.bottom = u32::max(offsets_max.bottom, offsets.bottom);
        }

        animation.dst_sprite_offset = offsets_max;
        animation.dst_sprite_size = size(offsets_max.left + offsets_max.right, offsets_max.top + offsets_max.bottom);

        let sprite_count = animation.src_offset.len() as u32;
        let total_width = (animation.dst_sprite_size.width * sprite_count) + (PADDING_PX * sprite_count);

        dst_width = u32::max(dst_width, total_width);
        dst_height += animation.dst_sprite_size.height + PADDING_PX;
    }

    actor.dst_image_info = actor.src_image_info.clone();
    actor.dst_image_info.width = dst_width;
    actor.dst_image_info.height = dst_height;
    actor.dst_image_info.line_size = pixel_size * (dst_width as usize);

    let dst_buffer_size = (dst_width as usize) * (dst_height as usize) * pixel_size;
    actor.dst_bytes = vec![0; dst_buffer_size];
}

fn compute_sprites_dst(actor: &mut Actor) {
    let mut x = 0;
    let mut y = 0;

    for animation in actor.animations.iter_mut() {
        let sprite_count = animation.src_rect.len();
        let dst_size = animation.dst_sprite_size;
        let dst_offset = animation.dst_sprite_offset;

        animation.dst_sprite_start = Offset { x, y };

        for sprite_index in 0..sprite_count {
            let src = animation.src_rect[sprite_index];
            let off = animation.src_offset[sprite_index];
            let width = src.width();
            let height = src.height();

            let x_loc = x + (dst_offset.left - off.left);
            let y_loc = y + (dst_offset.top - off.top);

            animation.dst_rect.push(Rect {
                left: x_loc,
                top: y_loc,
                right: x_loc + width,
                bottom: y_loc + height,
            });

            x += dst_size.width + PADDING_PX;
        }

        x = 0;
        y += dst_size.height + PADDING_PX;
    }
}

fn copy_sprite<P: Copy+Default+PartialEq>(actor: &mut Actor, animation_index: usize, sprite_index: usize) {
    let animation = &actor.animations[animation_index];
    let src = animation.src_rect[sprite_index];
    let dst = animation.dst_rect[sprite_index];
    assert!(src.height() == dst.height() && src.width() == dst.width(), "src and dst size must match");

    let src_pixel = actor.src_bytes.as_ptr() as *const P;
    let dst_pixel = actor.dst_bytes.as_mut_ptr() as *mut P;

    let scanline_src = actor.src_image_info.line_size / size_of::<P>();
    let scanline_dst = actor.dst_image_info.line_size / size_of::<P>();

    let height = src.height() as usize;
    let count = src.width() as usize;

    for y in 0..height {
        let y_src = (src.top as usize) + y;
        let y_dst = (dst.top as usize) + y;

        let src_offset = (y_src*scanline_src) + (src.left as usize);
        let dst_offset = (y_dst*scanline_dst) + (dst.left as usize);

        unsafe {
            let src = src_pixel.add(src_offset);
            let dst = dst_pixel.add(dst_offset);
            ::std::ptr::copy_nonoverlapping::<P>(src, dst, count);
        }
    }
}

fn copy_sprites_dst<P: Copy+Default+PartialEq>(actor: &mut Actor) {
    let animation_count = actor.animations.len();
    for animation_index in 0..animation_count {
        let sprite_count = actor.animations[animation_index].src_rect.len();
        for sprite_index in 0..sprite_count {
            copy_sprite::<P>(actor, animation_index, sprite_index);
        }
    }
}

//
// Export actor dst
//

fn export_image(actor: &mut Actor) -> Result<(), Box<dyn ::std::error::Error>> {
    use std::io::BufWriter;

    println!("Writing sprites data to {:?}", &actor.dst_path);

    let file = File::create(&actor.dst_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, actor.dst_image_info.width, actor.dst_image_info.height);
    encoder.set_compression(png::Compression::Best);
    encoder.set_color(actor.dst_image_info.color_type);
    encoder.set_depth(actor.dst_image_info.bit_depth);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    let source_chromaticities = png::SourceChromaticities::new(
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(&actor.dst_bytes)?;

    Ok(())
}

fn export_json(actor: &Actor) -> Result<(), Box<dyn ::std::error::Error>> {
    use std::io::Write;

    fn write_line<V: ::std::fmt::Debug>(js_out: &mut String, key: &str, value: V, last: bool) {
        js_out.push_str(&format!("      {:?}: {:?}", key, value));
        match last {
            true => js_out.push_str("\r\n"),
            false => js_out.push_str(",\r\n"),
        }
    }

    let default_buffer_size = 10_000;
    let mut js_out = String::with_capacity(default_buffer_size);

    js_out.push_str("{\r\n");
    js_out.push_str(&format!("  \"name\": {:?},\r\n", actor.file_name));
    js_out.push_str(&format!("  \"asset\": {:?},\r\n", actor.file_name.to_ascii_lowercase()));
    js_out.push_str(&"  \"animations\": [\r\n");

    let animation_count = actor.animations.len();
    for animation_index in 0..animation_count {
        let animation = &actor.animations[animation_index];
        let sprite_count = animation.dst_rect.len();
        let name = actor.animation_names.get(animation_index).map(|n| *n).unwrap_or_else(|| format!("UNNAMED_{animation_index}").leak() );

        js_out.push_str(&"    {\r\n");
        write_line(&mut js_out, "name", name, false);
        write_line(&mut js_out, "count", sprite_count, false);
        write_line(&mut js_out, "padding", PADDING_PX, false);
        write_line(&mut js_out, "x", animation.dst_sprite_start.x, false);
        write_line(&mut js_out, "y", animation.dst_sprite_start.y, false);
        write_line(&mut js_out, "width", animation.dst_sprite_size.width, false);
        write_line(&mut js_out, "height", animation.dst_sprite_size.height, true);

        if animation_index != animation_count - 1{
            js_out.push_str("    },\r\n");
        } else {
            js_out.push_str("    }\r\n");
        }
    }

    js_out.push_str("  ]\r\n}");

    let mut file = File::create(&actor.dst_path_json)?;
    file.write(js_out.as_bytes())?;

    Ok(())
}

//
// Processing actors
//

fn process_actor_sprites(state: &mut AssetsState) {
    fn process_single_actor<P: Copy+Default+PartialEq>(actor: &mut Actor) {
        find_actor_animation_sources::<P>(actor);
        prepare_actor_dst::<P>(actor);
        compute_sprites_dst(actor);
        copy_sprites_dst::<P>(actor);
    }

    for actor in state.actors.iter_mut() {
        if !actor.loaded {
            continue;
        }

        match (actor.src_image_info.bit_depth, actor.src_image_info.color_type) {
            (png::BitDepth::Eight, png::ColorType::Rgba) => process_single_actor::<[u8;4]>(actor),
            combined => unimplemented!("process_actor_sprites for {:?} is not implemented", combined)
        }

        if let Err(e) = export_image(actor) {
            println!("ERROR: Failed to export actors sprites: {:?}", e);
        }
        
        if let Err(e) = export_json(actor) {
            println!("ERROR: Failed to export actors sprites json: {:?}", e);
        }
    }
}

pub fn generate_sprites(filters: &[String]) {
    let mut state = AssetsState::default();
    load_actors(filters, &mut state);
    process_actor_sprites(&mut state);
}
