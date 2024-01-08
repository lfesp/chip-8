use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCALE_FACTOR: u32 = 12;

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video_subsystem = match sdl_context.video() {
            Ok(video) => video,
            Err(err) => panic!(
                "Could not obtain handle to the video subsystem! SDL_Error: {}",
                err
            ),
        };

        let window = video_subsystem
            .window(
                "rust-sdl2 Chip-8",
                SCREEN_WIDTH as u32 * SCALE_FACTOR,
                SCREEN_HEIGHT as u32 * SCALE_FACTOR,
            )
            //.window("rust-sdl2 demo: Video", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Display { canvas: canvas }
    }

    pub fn draw(&mut self, screen: &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
        self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas
            .set_draw_color(pixels::Color::RGB(255, 255, 255));

        for (y, row) in screen.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                if !pixel {
                    continue;
                }

                let _ = self.canvas.fill_rect(Rect::new(
                    (x as u32 * SCALE_FACTOR) as i32,
                    (y as u32 * SCALE_FACTOR) as i32,
                    SCALE_FACTOR,
                    SCALE_FACTOR,
                ));
            }
        }
        self.canvas.present();
    }
}
