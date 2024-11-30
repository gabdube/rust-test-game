//! Generate optimized sprites from the data in `assets/dev/tiny_sword`
//! Call this script using `cargo run -p loomz-tools --release -- -c generate_sprites -f [optional_filters]`
use std::fs::File;

const SRC_ROOT: &str = "assets/dev/tiny_sword/";
const DST_ROOT: &str = "assets/dev/textures/";

#[derive(Debug, Default, Copy, Clone)]
struct Size {
    pub width: u32,
    pub height: u32
}

impl Size {
    #[inline(always)]
    fn splat(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
struct SpriteRect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl SpriteRect {
    #[inline(always)]
    fn width(&self) -> u32 {
        self.right - self.left
    }

    #[inline(always)]
    fn height(&self) -> u32 {
        self.bottom - self.top
    }
}

#[derive(Debug, Default)]
struct Animation {
    pub dst_size: Size,
    pub src_sprites: Vec<SpriteRect>,
}

struct SpriteSheetInfo {
    pub cell_width: u32,
    pub cell_height: u32,
}

struct PngAsset {
    path: String,
    spritesheet_info: SpriteSheetInfo,
    image_info: png::OutputInfo,
    image_bytes: Vec<u8>,
}

#[derive(Default)]
struct AssetsState {
    actors: Vec<PngAsset>
}

fn spritesheet(cell_width: u32, cell_height: u32) -> SpriteSheetInfo {
    SpriteSheetInfo { cell_width, cell_height }
}

fn asset(path: &str, spritesheet_info: SpriteSheetInfo) -> PngAsset {
    PngAsset {
        path: format!("{SRC_ROOT}{path}"),
        spritesheet_info,
        image_info: png::OutputInfo { width: 0, height: 0, color_type: png::ColorType::Rgba, bit_depth: png::BitDepth::Eight, line_size: 0 },
        image_bytes: Vec::new(),
    }
}

fn load_state() -> AssetsState {
    let mut state = AssetsState::default();
    state.actors.push(asset("Factions/Knights/Troops/Pawn/Blue/Pawn_Blue.png", spritesheet(192, 192)));
    state.actors.push(asset("Factions/Knights/Troops/Warrior/Blue/Warrior_Blue.png", spritesheet(192, 192)));
    state
}

fn load_state_data(state: &mut AssetsState, filters: &[String]) {
    for actor in state.actors.iter_mut() {
        if filters.len() > 0 {
            if !filters.iter().any(|f| actor.path.matches(f).next().is_some() ) {
                continue;
            }
        }

        println!("Loading {:?}", actor.path);
        let decoder = png::Decoder::new(File::open(&actor.path).unwrap());
        let mut reader = decoder.read_info().unwrap();
        actor.image_bytes = vec![0; reader.output_buffer_size()];
        actor.image_info = reader.next_frame(&mut actor.image_bytes).unwrap();
    }
}

fn find_sprite_bounds<P: Copy+Default+PartialEq>(image_bytes: &[u8], line_size: usize, x1: usize, x2: usize, y1: usize, y2: usize) -> Option<SpriteRect> {
    let mut sprite = SpriteRect { left: u32::MAX, right: 0, top: u32::MAX, bottom: 0 };
    let image_pixels = unsafe { image_bytes.align_to::<P>().1 };
    let line_size_pixels = line_size / size_of::<P>();
    let zero = P::default();

    for y in y1..y2 {
        for x in x1..x2 {
            let pixel_index = (y*line_size_pixels) + x;
            let pixel = image_pixels[pixel_index];
            if pixel != zero {
                sprite.left = u32::min(sprite.left, x as u32);
                sprite.right = u32::max(sprite.right, (x+1) as u32);
                sprite.top = u32::min(sprite.top, y as u32);
                sprite.bottom = u32::max(sprite.bottom, (y+1) as u32);
            }
        }
    }

    if sprite.left == u32::MAX {
        None
    } else {
        Some(sprite)
    }
}

fn parse_src_sprites(asset: &PngAsset, animation: &mut Animation, yrange: [usize; 2]) {
    let [y1, y2] = yrange;

    let line_size = asset.image_info.line_size;
    let sprites_count = asset.image_info.width / asset.spritesheet_info.cell_width;
    animation.src_sprites = Vec::with_capacity(sprites_count as usize);

    for i in 0..sprites_count {
        let x1 = (i*asset.spritesheet_info.cell_width) as usize;
        let x2 = ((i+1)*asset.spritesheet_info.cell_width) as usize;
        let sprite = match (asset.image_info.bit_depth, asset.image_info.color_type) {
            (png::BitDepth::Eight, png::ColorType::Rgba) => find_sprite_bounds::<[u8;4]>(&asset.image_bytes, line_size, x1, x2, y1, y2),
            combined => unimplemented!("Bit depth for {:?} is not implemented", combined)
        };

        if let Some(sprite) = sprite {
            animation.src_sprites.push(sprite);
        }
    }

    for sprite in animation.src_sprites.iter() {
        animation.dst_size.width = u32::max(animation.dst_size.width, sprite.width());
        animation.dst_size.height = u32::max(animation.dst_size.height, sprite.height());
    }
}

fn dst_path<'a>(asset: &'a PngAsset) -> Result<String, Box<dyn ::std::error::Error>> {
    let name = ::std::path::Path::new(&asset.path)
        .file_name()
        .and_then(|name| name.to_str() )
        .unwrap();

    let dst_path = format!("{DST_ROOT}{name}");
    if ::std::fs::exists(&dst_path)? {
        ::std::fs::remove_file(&dst_path)?;
    }

    Ok(dst_path)
}

fn pixel_size(asset: &PngAsset) -> usize {
    match (asset.image_info.bit_depth, asset.image_info.color_type) {
        (png::BitDepth::Eight, png::ColorType::Rgba) => size_of::<[u8;4]>(),
        combined => unimplemented!("Pixel size for {:?} is not implemented", combined)
    }
}

fn exported_image_size(animations: &[Animation]) -> Size {
    let mut dst_total_size = Size::default();
    for animation in animations {
        dst_total_size.width = u32::max(dst_total_size.width, animation.dst_size.width * (animation.src_sprites.len() as u32) );
        dst_total_size.height += animation.dst_size.height;
    }

    // Align dst image to 16 bytes (the block size for the BC7 format)
    // BLOCK_SIZE / PIXEL_SIZE == 16 / 4 == 4
    dst_total_size.width = dst_total_size.width + (4 - (dst_total_size.width % 4));
    dst_total_size.height = dst_total_size.height + (4 - (dst_total_size.height % 4));

    dst_total_size
}

fn copy_sprite<P2>(
    src_bytes: &[u8],
    dst_bytes: &mut [u8],
    src_line_size: usize,
    dst_line_size: usize,
    src_sprite: &SpriteRect,
    dst_sprite: &SpriteRect,
) {
    type P = [u8; 4];

    assert_eq!(src_sprite.width(), dst_sprite.width(), "src and dst must have the same size");
    assert_eq!(src_sprite.height(), dst_sprite.height(), "src and dst must have the same size");

    let src_pixel = unsafe { src_bytes.align_to::<P>().1 };
    let dst_pixel = unsafe { dst_bytes.align_to_mut::<P>().1 };

    let p_size = size_of::<P>();
    let src_line_pixel = src_line_size / p_size;
    let dst_line_pixel = dst_line_size / p_size;

    let width = src_sprite.width() as usize;
    let height = src_sprite.height() as usize;

    for y in 0..height {
        let src_y = (src_sprite.top as usize) + y;
        let src_offset = (src_y * src_line_pixel) + src_sprite.left as usize;

        let dst_y = (dst_sprite.top as usize) + y;
        let dst_offset = (dst_y * dst_line_pixel) + dst_sprite.left as usize;
        
        unsafe {
            ::std::ptr::copy_nonoverlapping::<P>(
                src_pixel.as_ptr().add(src_offset),
                dst_pixel.as_mut_ptr().add(dst_offset),
                width
            );
        }
    }
}

fn export_image_data(asset: &PngAsset, animations: &[Animation], dst_bytes: &mut [u8], dst_line_size: usize) {
    let src_line_size = asset.image_info.line_size;
    let dst_line_size = dst_line_size;

    let mut dst_sprite = SpriteRect::default();
    let mut dst_offset_y = 0;
    for animation in animations {
        let [dst_width, dst_height] = animation.dst_size.splat();
        let mut dst_offset_x = 0;

        for src_sprite in &animation.src_sprites {
            let offset_x = dst_width - src_sprite.width(); // If the sprite is smaller than the sprite size, it is offseted by the size difference
            let offset_y = dst_height - src_sprite.height();

            dst_sprite.left = dst_offset_x + offset_x;   
            dst_sprite.right = dst_sprite.left + src_sprite.width();

            dst_sprite.top = dst_offset_y + offset_y;
            dst_sprite.bottom = dst_sprite.top + src_sprite.height();

            match (asset.image_info.bit_depth, asset.image_info.color_type) {
                (png::BitDepth::Eight, png::ColorType::Rgba) => copy_sprite::<[u8; 4]>(&asset.image_bytes, dst_bytes, src_line_size, dst_line_size, &src_sprite, &dst_sprite),
                combined => unimplemented!("Bit depth for {:?} is not implemented", combined)
            };
            
            dst_offset_x += dst_width;
        }

        dst_offset_y += animation.dst_size.height;
    }
}

fn save_dst_image(asset: &PngAsset, dst_width: u32, dst_height: u32, data: &[u8]) -> Result<(), Box<dyn ::std::error::Error>> {
    use std::io::BufWriter;
    let dst = dst_path(asset)?;
    println!("Writing sprites data to {:?}", dst);

    let file = File::create(dst)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, dst_width, dst_height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(data)?;

    Ok(())
}

fn export_actor_sprites(asset: &PngAsset, animations: &[Animation]) {
    let dst_total_size = exported_image_size(animations);
    let dst_line_line = dst_total_size.width as usize * pixel_size(asset);
    let total_dst_data_size = dst_line_line * (dst_total_size.height as usize);
    let mut dst_data = vec![0u8; total_dst_data_size];

    export_image_data(asset, animations, &mut dst_data, dst_line_line);

    if let Err(e) = save_dst_image(asset, dst_total_size.width, dst_total_size.height, &dst_data) {
        println!("ERROR: Failed to export actors sprites: {:?}", e);
    }
}

fn generate_actors_sprites(state: &AssetsState) {
    for actor in state.actors.iter() {
        if actor.image_bytes.len() == 0 {
            continue;
        }

        let cell_height = actor.spritesheet_info.cell_height;
        let animations_count = actor.image_info.height / cell_height;
        let mut animations = Vec::with_capacity(animations_count as usize);
        
        for i in 0..animations_count {
            let yrange = [(i*cell_height) as usize, ((i+1)*cell_height) as usize];
            let mut animation = Animation::default();
            parse_src_sprites(actor, &mut animation, yrange);
            animations.push(animation);
        }

        export_actor_sprites(actor, &animations);
    }
}

pub fn generate_sprites(filters: &[String]) {
    let mut state = load_state();
    load_state_data(&mut state, filters);
    generate_actors_sprites(&state);
}
