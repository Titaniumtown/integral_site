use meval::Expr;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
mod misc;
use crate::misc::{Chart, DrawResult};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Manages Chart generation and caching of values
#[wasm_bindgen]
pub struct ChartManager {
    func_str: String,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    num_interval: usize,
    resolution: i32,
    back_cache: Option<Vec<(f32, f32)>>,
    front_cache: Option<(Vec<(f32, f32, f32)>, f32)>,
    use_back_cache: bool,
    use_front_cache: bool,
}

#[wasm_bindgen]
impl ChartManager {
    pub fn new(
        func_str: String, min_x: f32, max_x: f32, min_y: f32, max_y: f32, num_interval: usize,
        resolution: i32,
    ) -> Self {
        Self {
            func_str,
            min_x,
            max_x,
            min_y,
            max_y,
            num_interval,
            resolution,
            back_cache: None,
            front_cache: None,
            use_back_cache: false,
            use_front_cache: false,
        }
    }

    // Used in order to hook into `panic!()` to log in the browser's console
    pub fn init_panic_hook() { panic::set_hook(Box::new(console_error_panic_hook::hook)); }

    #[inline(always)]
    fn get_back_cache(&self) -> Vec<(f32, f32)> {
        log("Using back_cache");
        match &self.back_cache {
            Some(x) => x.clone(),
            None => panic!("use_back_cache is true, but back_cache is None!"),
        }
    }

    #[inline(always)]
    fn get_front_cache(&self) -> (Vec<(f32, f32, f32)>, f32) {
        log("Using front_cache");
        match &self.front_cache {
            Some(x) => x.clone(),
            None => panic!("use_front_cache is true, but front_cache is None!"),
        }
    }

    #[inline(always)]
    fn draw(
        &mut self, element: HtmlCanvasElement,
    ) -> DrawResult<(impl Fn((i32, i32)) -> Option<(f32, f32)>, f32)> {
        let expr: Expr = self.func_str.parse().unwrap();
        let func = expr.bind("x").unwrap();

        let backend = CanvasBackend::with_canvas_object(element).unwrap();
        let root = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .margin(20.0)
            .caption(format!("y={}", self.func_str), font)
            .x_label_area_size(30.0)
            .y_label_area_size(30.0)
            .build_cartesian_2d(self.min_x..self.max_x, self.min_y..self.max_y)?;

        chart.configure_mesh().x_labels(3).y_labels(3).draw()?;

        let absrange = (self.max_x - self.min_x).abs();
        let data: Vec<(f32, f32)> = match self.use_back_cache {
            true => self.get_back_cache(),
            false => {
                log("Updating back_cache");
                let output: Vec<(f32, f32)> = (1..=self.resolution)
                    .map(|x| ((x as f32 / self.resolution as f32) * absrange) + self.min_x)
                    .map(|x| (x, func(x as f64) as f32))
                    .filter(|(_, y)| &self.min_y <= y && y <= &self.max_y)
                    .collect();
                self.back_cache = Some(output.clone());
                output
            }
        };

        chart.draw_series(LineSeries::new(data, &RED))?;

        let (rect_data, area): (Vec<(f32, f32, f32)>, f32) = match self.use_front_cache {
            true => self.get_front_cache(),
            false => {
                log("Updating front_cache");
                let step = absrange / (self.num_interval as f32);
                let output: (Vec<(f32, f32, f32)>, f32) = self.integral_rectangles(step, &func);
                self.front_cache = Some(output.clone());
                output
            }
        };

        // Draw rectangles
        chart.draw_series(
            rect_data
                .iter()
                .map(|(x1, x2, y)| Rectangle::new([(*x2, *y), (*x1, 0.0)], &BLUE)),
        )?;

        root.present()?;
        Ok((chart.into_coord_trans(), area))
    }

    pub fn update(
        &mut self, canvas: HtmlCanvasElement, func_str: &str, min_x: f32, max_x: f32, min_y: f32,
        max_y: f32, num_interval: usize, resolution: i32,
    ) -> Result<Chart, JsValue> {
        let underlying_update = (*func_str != self.func_str)
            | (min_x != self.min_x)
            | (max_x != self.max_x)
            | (min_y != self.min_y)
            | (max_y != self.max_y);

        if underlying_update {
            if min_x >= max_x {
                panic!("min_x is greater than (or equal to) than max_x!");
            }

            if min_y >= max_y {
                panic!("min_y is greater than (or equal to) than max_y!");
            }
        }

        if 0 > resolution {
            panic!("resolution cannot be less than 0");
        }

        self.use_back_cache =
            !underlying_update && self.resolution == resolution && self.back_cache.is_some();
        self.use_front_cache =
            !underlying_update && num_interval == self.num_interval && self.front_cache.is_some();

        self.func_str = func_str.to_string();
        self.min_x = min_x;
        self.max_x = max_x;
        self.min_y = min_y;
        self.max_y = max_y;
        self.num_interval = num_interval;
        self.resolution = resolution;

        let draw_output = self.draw(canvas).map_err(|err| err.to_string())?;
        let map_coord = draw_output.0;

        let chart = Chart {
            convert: Box::new(move |coord| map_coord(coord).map(|(x, y)| (x, y))),
            area: draw_output.1,
        };

        Ok(chart)
    }

    // Creates and does the math for creating all the rectangles under the graph
    #[inline(always)]
    fn integral_rectangles(
        &self, step: f32, func: &dyn Fn(f64) -> f64,
    ) -> (Vec<(f32, f32, f32)>, f32) {
        let data2: Vec<(f32, f32, f32)> = (0..self.num_interval)
            .map(|e| {
                let x: f32 = ((e as f32) * step) + self.min_x;

                // Makes sure rectangles are properly handled on x values below 0
                let x2: f32 = match x > 0.0 {
                    true => x + step,
                    false => x - step,
                };

                let tmp1: f32 = func(x as f64) as f32;
                let tmp2: f32 = func(x2 as f64) as f32;

                // Chooses the y value who's absolute value is the smallest
                let y: f32 = match tmp2.abs() > tmp1.abs() {
                    true => tmp1,
                    false => tmp2,
                };

                if !y.is_nan() {
                    (x, x2, y)
                } else {
                    (0.0, 0.0, 0.0)
                }
            })
            .filter(|ele| ele != &(0.0, 0.0, 0.0))
            .collect();
        let area: f32 = data2.iter().map(|(_, _, y)| y * step).sum(); // sum of all rectangles' areas
        (data2, area)
    }
}
