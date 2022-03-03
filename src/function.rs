#![allow(clippy::too_many_arguments)] // Clippy, shut

#[allow(unused_imports)]
use crate::misc::debug_log;

use eframe::egui::{
    plot::{BarChart, Line, Value, Values},
    widgets::plot::Bar,
};
use meval::Expr;
use std::fmt::{self, Debug};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RiemannSum {
    Left,
    Middle,
    Right,
}

impl fmt::Display for RiemannSum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

pub struct Function {
    function: Box<dyn Fn(f64) -> f64>,
    func_str: String,
    min_x: f64,
    max_x: f64,
    pixel_width: usize,

    back_cache: Option<Vec<Value>>,
    front_cache: Option<(Vec<Bar>, f64)>,

    pub(crate) integral: bool,
    integral_min_x: f64,
    integral_max_x: f64,
    integral_num: usize,
    sum: RiemannSum,
}

impl Function {
    pub fn new(
        func_str: String, min_x: f64, max_x: f64, pixel_width: usize, integral: bool,
        integral_min_x: Option<f64>, integral_max_x: Option<f64>, integral_num: Option<usize>,
        sum: Option<RiemannSum>,
    ) -> Self {
        // Makes sure proper arguments are passed when integral is enabled
        if integral {
            integral_min_x
                .expect("Invalid arguments: integral_min_x is None, but integral is enabled.");
            integral_max_x
                .expect("Invalid arguments: integral_max_x is None, but integral is enabled.");
            integral_num
                .expect("Invalid arguments: integral_num is None, but integral is enabled.");
            sum.expect("Invalid arguments: sum is None, but integral is enabled");
        }

        let expr: Expr = func_str.parse().unwrap();
        let func = expr.bind("x").unwrap();
        Self {
            function: Box::new(func),
            func_str,
            min_x,
            max_x,
            pixel_width,
            back_cache: None,
            front_cache: None,
            integral,
            integral_min_x: integral_min_x.unwrap_or(f64::NAN),
            integral_max_x: integral_max_x.unwrap_or(f64::NAN),
            integral_num: integral_num.unwrap_or(0),
            sum: sum.unwrap_or(RiemannSum::Left),
        }
    }

    // Runs the internal function to get values
    fn run_func(&self, x: f64) -> f64 { (self.function)(x) }

    pub fn update(
        &mut self, func_str: String, integral: bool, integral_min_x: Option<f64>,
        integral_max_x: Option<f64>, integral_num: Option<usize>, sum: Option<RiemannSum>,
    ) {
        if func_str.is_empty() {
            self.func_str = func_str;
            return;
        }

        // If the function string changes, just wipe and restart from scratch
        if func_str != self.func_str {
            *self = Self::new(
                func_str,
                self.min_x,
                self.max_x,
                self.pixel_width,
                integral,
                integral_min_x,
                integral_max_x,
                integral_num,
                sum,
            );
            return;
        }

        self.integral = integral;

        // Makes sure proper arguments are passed when integral is enabled
        if integral
            && (integral_min_x != Some(self.integral_min_x))
                | (integral_max_x != Some(self.integral_max_x))
                | (integral_num != Some(self.integral_num))
                | (sum != Some(self.sum))
        {
            self.front_cache = None;
            self.integral_min_x = integral_min_x.expect("integral_min_x is None");
            self.integral_max_x = integral_max_x.expect("integral_max_x is None");
            self.integral_num = integral_num.expect("integral_num is None");
            self.sum = sum.expect("sum is None");
        }
    }

    pub fn update_bounds(&mut self, min_x: f64, max_x: f64, pixel_width: usize) {
        if pixel_width != self.pixel_width {
            self.back_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        } else if ((min_x != self.min_x) | (max_x != self.max_x)) && self.back_cache.is_some() {
            let resolution: f64 = self.pixel_width as f64 / (max_x.abs() + min_x.abs());
            let back_cache = self.back_cache.as_ref().unwrap();

            let x_data: Vec<f64> = back_cache.iter().map(|ele| ele.x).collect();

            self.back_cache = Some(
                (0..=self.pixel_width)
                    .map(|x| (x as f64 / resolution as f64) + min_x)
                    .map(|x| {
                        // If x is outside of previous bounds, just go ahead and just skip searching for the index
                        if (x < self.min_x) | (self.max_x < x) {
                            return Value::new(x, self.run_func(x));
                        }

                        let i_option = x_data.iter().position(|&r| r == x); // Optimize this later, this could be done much much better, but tbh it doesn't matter that much as the program is already super fast

                        if let Some(i) = i_option {
                            back_cache[i]
                        } else {
                            Value::new(x, self.run_func(x))
                        }
                    })
                    .collect(),
            );
        } else {
            self.back_cache = None;
            self.min_x = min_x;
            self.max_x = max_x;
            self.pixel_width = pixel_width;
        }
    }

    pub fn run(&mut self) -> (Line, Option<(BarChart, f64)>) {
        let back_values: Line = Line::new(Values::from_values({
            if self.back_cache.is_none() {
                let resolution: f64 =
                    (self.pixel_width as f64 / (self.max_x - self.min_x).abs()) as f64;
                self.back_cache = Some(
                    (0..=self.pixel_width)
                        .map(|x| (x as f64 / resolution as f64) + self.min_x)
                        .map(|x| Value::new(x, self.run_func(x)))
                        .collect(),
                );
            }

            self.back_cache.as_ref().unwrap().clone()
        }));

        if self.integral {
            let front_bars: (BarChart, f64) = {
                if self.front_cache.is_none() {
                    let (data, area) = self.integral_rectangles();
                    self.front_cache =
                        Some((data.iter().map(|(x, y)| Bar::new(*x, *y)).collect(), area));
                }
                let cache = self.front_cache.as_ref().unwrap();
                (BarChart::new(cache.0.clone()), cache.1)
            };

            (back_values, Some(front_bars))
        } else {
            (back_values, None)
        }
    }

    // Creates and does the math for creating all the rectangles under the graph
    fn integral_rectangles(&self) -> (Vec<(f64, f64)>, f64) {
        if self.integral_min_x.is_nan() {
            panic!("integral_min_x is NaN")
        } else if self.integral_max_x.is_nan() {
            panic!("integral_max_x is NaN")
        }

        let step = (self.integral_min_x - self.integral_max_x).abs() / (self.integral_num as f64);

        let half_step = step / 2.0;
        let data2: Vec<(f64, f64)> = (0..self.integral_num)
            .map(|e| {
                let x: f64 = ((e as f64) * step) + self.integral_min_x;
                let x2: f64 = match x.is_sign_positive() {
                    true => x + step,
                    false => x - step,
                };

                let (left_x, right_x) = match x.is_sign_positive() {
                    true => (x, x2),
                    false => (x2, x),
                };

                (
                    match x.is_sign_positive() {
                        true => x + half_step,
                        false => x - half_step,
                    },
                    match self.sum {
                        RiemannSum::Left => self.run_func(left_x),
                        RiemannSum::Right => self.run_func(right_x),
                        RiemannSum::Middle => {
                            (self.run_func(left_x) + self.run_func(right_x)) / 2.0
                        }
                    },
                )
            })
            .filter(|(_, y)| !y.is_nan())
            .collect();
        let area: f64 = data2.iter().map(|(_, y)| y * step).sum(); // sum of all rectangles' areas
        (data2, area)
    }

    pub fn empty_func_str(&mut self) { self.func_str = String::new(); }
}
