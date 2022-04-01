use crate::consts::*;
use crate::function::{FunctionEntry, Riemann, DEFAULT_FUNCTION_ENTRY};
use crate::misc::{dyn_mut_iter, option_vec_printer, JsonFileOutput, SerdeValueHelper};
use eframe::{egui, epi};
use egui::{
	plot::Plot, Button, CentralPanel, Color32, ComboBox, Context, FontData, FontDefinitions,
	FontFamily, Key, RichText, SidePanel, Slider, TopBottomPanel, Vec2, Visuals, Window,
};
use instant::Duration;
use std::collections::HashMap;
use std::{collections::BTreeMap, io::Read, ops::BitXorAssign, str};

#[cfg(threading)]
use rayon::iter::{IndexedParallelIterator, ParallelIterator};

// Stores data loaded from files
struct Assets {
	// Stores `FontDefinitions`
	pub fonts: FontDefinitions,

	// Help blurbs
	pub text_help_expr: String,
	pub text_help_vars: String,
	pub text_help_panel: String,
	pub text_help_function: String,
	pub text_help_other: String,

	// Explanation of license
	pub text_license_info: String,
}

impl Assets {
	pub fn new(fonts: FontDefinitions, json: JsonFileOutput) -> Self {
		Self {
			fonts,
			text_help_expr: json.help_expr,
			text_help_vars: json.help_vars,
			text_help_panel: json.help_panel,
			text_help_function: json.help_function,
			text_help_other: json.help_other,
			text_license_info: json.license_info,
		}
	}

	#[cfg(test)] // Only used for testing
	pub fn get_json_file_output(&self) -> JsonFileOutput {
		JsonFileOutput {
			help_expr: self.text_help_expr.clone(),
			help_vars: self.text_help_vars.clone(),
			help_panel: self.text_help_panel.clone(),
			help_function: self.text_help_function.clone(),
			help_other: self.text_help_other.clone(),
			license_info: self.text_license_info.clone(),
		}
	}
}

lazy_static::lazy_static! {
	// Load all of the data from the compressed tarball
	static ref ASSETS: Assets = {
		let start = instant::Instant::now();

		tracing::info!("Loading assets...");
		let mut tar_file_data = Vec::new();
		let _ = ruzstd::StreamingDecoder::new(&mut include_bytes!("../assets.tar.zst").as_slice()).expect("failed to decompress assets").read_to_end(&mut tar_file_data).expect("failed to read assets");

		let mut tar_archive = tar::Archive::new(&*tar_file_data);

		// Stores fonts
		let mut font_ubuntu_light: Option<FontData> = None;
		let mut font_notoemoji: Option<FontData> = None;
		let mut font_hack: Option<FontData> = None;

		// Stores text
		let mut text_data: Option<JsonFileOutput> = None;


		tracing::info!("Reading assets...");
		// Iterate through all entries in the tarball
		for file in tar_archive.entries().unwrap() {
			let mut file = file.unwrap();
			let mut data: Vec<u8> = Vec::new();
			file.read_to_end(&mut data).unwrap();
			let path = file.header().path().unwrap();
			let path_string = path.to_string_lossy();

			tracing::debug!("Loading file: {}", path_string);

			// Match the file extention
			if path_string.ends_with(".ttf") {
				// Parse font files
				let font_data = FontData::from_owned(data);
				match path_string.as_ref() {
					"Hack-Regular.ttf" => {
						font_hack = Some(font_data);
					},
					"NotoEmoji-Regular.ttf" => {
						font_notoemoji = Some(font_data);
					},
					"Ubuntu-Light.ttf" => {
						font_ubuntu_light = Some(font_data);
					},
					_ => {
						panic!("Font File {} not expected!", path_string);
					}
				}
			} else if path_string.ends_with(".json") {
				// Parse json file
				let string_data = str::from_utf8(&data).unwrap().to_string();
				match path_string.as_ref() {
					"text.json" => {
						text_data = Some(SerdeValueHelper::new(&string_data).parse_values());
					},
					_ => {
						panic!("Json file {} not expected!", path_string);
					}
				}
			} else {
				panic!("Other file {} not expected!", path_string);
			}
		}

		tracing::info!("Done loading assets! Took: {:?}", start.elapsed());

		let mut font_data: BTreeMap<String, FontData> = BTreeMap::new();
		let mut families = BTreeMap::new();

		font_data.insert("Hack".to_owned(), font_hack.expect("Hack font not found!"));
		font_data.insert("Ubuntu-Light".to_owned(), font_ubuntu_light.expect("Ubuntu Light font not found!"));
		font_data.insert("NotoEmoji-Regular".to_owned(), font_notoemoji.expect("Noto Emoji font not found!"));

		families.insert(
			FontFamily::Monospace,
			vec![
				"Hack".to_owned(),
				"Ubuntu-Light".to_owned(),
				"NotoEmoji-Regular".to_owned(),
			],
		);

		families.insert(
			FontFamily::Proportional,
			vec!["Ubuntu-Light".to_owned(), "NotoEmoji-Regular".to_owned()],
		);

		let fonts = FontDefinitions {
			font_data,
			families,
		};


		// Create and return Assets struct
		Assets::new(
			fonts, text_data.expect("Text data not found!"))
	};
}

/// Tests to make sure archived (and compressed) assets match expected data
#[test]
fn test_file_data() {
	let mut font_data: BTreeMap<String, FontData> = BTreeMap::new();
	let mut families = BTreeMap::new();

	font_data.insert(
		"Hack".to_owned(),
		FontData::from_owned(include_bytes!("../assets/Hack-Regular.ttf").to_vec()),
	);
	font_data.insert(
		"Ubuntu-Light".to_owned(),
		FontData::from_owned(include_bytes!("../assets/Ubuntu-Light.ttf").to_vec()),
	);
	font_data.insert(
		"NotoEmoji-Regular".to_owned(),
		FontData::from_owned(include_bytes!("../assets/NotoEmoji-Regular.ttf").to_vec()),
	);

	families.insert(
		FontFamily::Monospace,
		vec![
			"Hack".to_owned(),
			"Ubuntu-Light".to_owned(),
			"NotoEmoji-Regular".to_owned(),
		],
	);

	families.insert(
		FontFamily::Proportional,
		vec!["Ubuntu-Light".to_owned(), "NotoEmoji-Regular".to_owned()],
	);

	let fonts = FontDefinitions {
		font_data,
		families,
	};

	assert_eq!(ASSETS.fonts, fonts);

	let json_data: SerdeValueHelper = SerdeValueHelper::new(include_str!("../assets/text.json"));

	let asset_json = ASSETS.get_json_file_output();
	let json_data_parsed = json_data.parse_values();

	assert_eq!(asset_json, json_data_parsed);

	// NOTE: UPDATE THIS STRING IF `license_info` IN `text.json` IS MODIFIED
	let target_license_info = "The AGPL license ensures that the end user, even if not hosting the program itself, is still guaranteed access to the source code of the project in question.";

	assert_eq!(target_license_info, asset_json.license_info);
	assert_eq!(target_license_info, json_data_parsed.license_info);
}

cfg_if::cfg_if! {
	if #[cfg(target_arch = "wasm32")] {
		use wasm_bindgen::JsCast;

		/// Removes the "loading" element on the web page that displays a loading indicator
		fn stop_loading() {
			let document = web_sys::window().expect("Could not get web_sys window").document().expect("Could not get web_sys document");

			let loading_element = document.get_element_by_id("loading").expect("Couldn't get loading indicator element")
			.dyn_into::<web_sys::HtmlElement>().unwrap();

			// Remove the element
			loading_element.remove();
		}
	}
}

/// Stores current settings/state of [`MathApp`]
// TODO: find a better name for this
#[derive(Copy, Clone)]
pub struct AppSettings {
	/// Stores the type of Rienmann sum that should be calculated
	pub riemann_sum: Riemann,

	/// Min and Max range for calculating an integral
	pub integral_min_x: f64,

	/// Max value for calculating an
	pub integral_max_x: f64,

	/// Stores whether or not integral settings have changed
	pub integral_changed: bool,

	/// Number of rectangles used to calculate integral
	pub integral_num: usize,

	/// Stores whether or not displaying extrema is enabled
	pub do_extrema: bool,

	/// Stores whether or not displaying roots is enabled
	pub do_roots: bool,

	/// Stores current plot pixel width
	pub plot_width: usize,
}

impl Default for AppSettings {
	/// Default implementation of `AppSettings`, this is how the application
	/// starts up
	fn default() -> Self {
		Self {
			riemann_sum: DEFAULT_RIEMANN,
			integral_min_x: DEFAULT_MIN_X,
			integral_max_x: DEFAULT_MAX_X,
			integral_changed: true,
			integral_num: DEFAULT_INTEGRAL_NUM,
			do_extrema: true,
			do_roots: true,
			plot_width: 0,
		}
	}
}

/// The actual application
pub struct MathApp {
	/// Stores vector of functions
	functions: Vec<FunctionEntry>,

	/// Stores vector containing the string representation of the functions.
	/// This is used because of hacky reasons
	func_strs: Vec<String>,

	/// Stores last error from parsing functions (used to display the same error
	/// when side panel is minimized)
	func_errors: Vec<Option<(usize, String)>>,

	/// Stores whether or not an error is stored in `self.func_errors`
	exists_error: bool,

	/// Contains the list of Areas calculated (the vector of f64) and time it
	/// took for the last frame (the Duration). Stored in a Tuple.
	last_info: (Vec<Option<f64>>, Duration),

	/// Stores whether or not dark mode is enabled
	dark_mode: bool,

	/// Stores whether or not the text boxes are focused
	text_boxes_focused: bool,

	/// Stores opened windows/elements for later reference
	opened: HashMap<&'static str, bool>,

	/// Stores settings (pretty self-explanatory)
	settings: AppSettings,
}

impl Default for MathApp {
	fn default() -> Self {
		Self {
			functions: vec![DEFAULT_FUNCTION_ENTRY.clone()],
			func_strs: vec![String::new()],
			func_errors: vec![None],
			exists_error: false,
			last_info: (vec![None], Duration::ZERO),
			dark_mode: true,
			text_boxes_focused: false,
			opened: HashMap::from([("help", true), ("info", false), ("side_panel", true)]),
			settings: AppSettings::default(),
		}
	}
}

impl MathApp {
	#[allow(dead_code)] // This is used lol
	/// Create new instance of [`MathApp`] and return it
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// Remove loading indicator on wasm
		#[cfg(target_arch = "wasm32")]
		stop_loading();

		#[cfg(threading)]
		tracing::info!("Threading: Enabled");

		#[cfg(not(threading))]
		tracing::info!("Threading: Disabled");

		tracing::info!(
			"Integration name: {} Url: {:?}",
			cc.integration_info.name,
			if let Some(url) = &cc.integration_info.web_info {
				&url.location.url
			} else {
				"N/A"
			}
		);

		tracing::info!("egui app initialized.");
		Self::default() // initialize `MathApp`
	}

	fn get_opened_mut(&mut self, id: &str) -> &mut bool { self.opened.get_mut(id).unwrap() }

	fn get_opened(&self, id: &str) -> bool { *self.opened.get(id).unwrap() }

	/// Creates SidePanel which contains configuration options
	fn side_panel(&mut self, ctx: &Context) {
		// Side Panel which contains vital options to the operation of the application
		// (such as adding functions and other options)
		SidePanel::left("side_panel")
			.resizable(false)
			.show(ctx, |ui| {
				let prev_sum = self.settings.riemann_sum;
				// ComboBox for selecting what Riemann sum type to use
				ComboBox::from_label("Riemann Sum Type")
					.selected_text(self.settings.riemann_sum.to_string())
					.show_ui(ui, |ui| {
						ui.selectable_value(&mut self.settings.riemann_sum, Riemann::Left, "Left");
						ui.selectable_value(
							&mut self.settings.riemann_sum,
							Riemann::Middle,
							"Middle",
						);
						ui.selectable_value(
							&mut self.settings.riemann_sum,
							Riemann::Right,
							"Right",
						);
					});
				let riemann_changed = prev_sum == self.settings.riemann_sum;

				// Config options for Extrema and roots
				let mut extrema_toggled: bool = false;
				let mut roots_toggled: bool = false;
				ui.horizontal(|ui| {
					extrema_toggled = ui
						.add(Button::new("Extrema"))
						.on_hover_text(match self.settings.do_extrema {
							true => "Disable Displaying Extrema",
							false => "Display Extrema",
						})
						.clicked();

					roots_toggled = ui
						.add(Button::new("Roots"))
						.on_hover_text(match self.settings.do_roots {
							true => "Disable Displaying Roots",
							false => "Display Roots",
						})
						.clicked();
				});

				// If options toggled, flip the boolean
				self.settings.do_extrema.bitxor_assign(extrema_toggled);
				self.settings.do_roots.bitxor_assign(roots_toggled);

				let min_x_old = self.settings.integral_min_x;
				let min_x_changed = ui
					.add(
						Slider::new(&mut self.settings.integral_min_x, INTEGRAL_X_RANGE)
							.text("Min X"),
					)
					.changed();

				let max_x_old = self.settings.integral_max_x;
				let max_x_changed = ui
					.add(
						Slider::new(&mut self.settings.integral_max_x, INTEGRAL_X_RANGE)
							.text("Max X"),
					)
					.changed();

				// Checks integral bounds, and if they are invalid, fix them
				if self.settings.integral_min_x >= self.settings.integral_max_x {
					if max_x_changed {
						self.settings.integral_max_x = max_x_old;
					} else if min_x_changed {
						self.settings.integral_min_x = min_x_old;
					} else {
						// No clue how this would happen, but just in case
						self.settings.integral_min_x = DEFAULT_MIN_X;
						self.settings.integral_max_x = DEFAULT_MAX_X;
					}
				}

				// Number of Rectangles for Riemann sum
				let integral_num_changed = ui
					.add(
						Slider::new(&mut self.settings.integral_num, INTEGRAL_NUM_RANGE)
							.text("Interval"),
					)
					.changed();

				self.settings.integral_changed =
					max_x_changed | min_x_changed | integral_num_changed | riemann_changed;

				let functions_len = self.functions.len();
				let mut remove_i: Option<usize> = None;
				self.text_boxes_focused = false;
				self.exists_error = false;
				for (i, function) in self.functions.iter_mut().enumerate() {
					// Entry for a function
					ui.horizontal(|ui| {
						ui.label("Function:");

						// There's more than 1 function! Functions can now be deleted
						if ui
							.add_enabled(functions_len > 1, Button::new("X"))
							.on_hover_text("Delete Function")
							.clicked()
						{
							remove_i = Some(i);
						}

						// Toggle integral being enabled or not
						function.integral.bitxor_assign(
							ui.add(Button::new("∫"))
								.on_hover_text(match function.integral {
									true => "Don't integrate",
									false => "Integrate",
								})
								.clicked(),
						);

						// Toggle showing the derivative (even though it's already calculated this
						// option just toggles if it's displayed or not)
						function.derivative.bitxor_assign(
							ui.add(Button::new("d/dx"))
								.on_hover_text(match function.derivative {
									true => "Don't Differentiate",
									false => "Differentiate",
								})
								.clicked(),
						);

						// Contains the function string in a text box that the user can edit
						let (focused, changed, error) =
							function.auto_complete(ui, &mut self.func_strs[i]);
						if focused {
							self.text_boxes_focused = true;
						}

						if error.is_some() {
							self.exists_error = true;
							if changed {
								self.func_errors[i] =
									function.get_test_result().map(|error| (i, error));
							}
						}
					});

					// let func_failed = self.func_errors[i].is_some();
					// if func_failed {
					//
					// }

					// let update_result = function
					// 	.update(&self.func_strs[i], integral_enabled,
					// derivative_enabled) 	.map(|error| (i, error));

					// if update_result.is_some() {
					// 	self.exists_error = true;
					// }

					// self.func_errors[i] = update_result
				}

				// Remove function if the user requests it
				if let Some(remove_i_unwrap) = remove_i {
					self.functions.remove(remove_i_unwrap);
					self.func_strs.remove(remove_i_unwrap);
					self.func_errors.remove(remove_i_unwrap);
				}

				// Hyperlink to project's github
				ui.hyperlink_to(
					"I'm Open Source!",
					"https://github.com/Titaniumtown/YTBN-Graphing-Software",
				);

				// Licensing information
				ui.label(RichText::new("(and licensed under AGPLv3)").color(Color32::LIGHT_GRAY))
					.on_hover_text(&ASSETS.text_license_info);
			});
	}
}

impl epi::App for MathApp {
	/// Called each time the UI needs repainting, which may be many times per
	/// second.
	fn update(&mut self, ctx: &Context, _frame: &mut epi::Frame) {
		// start timer
		let start = instant::Instant::now();

		// Set dark/light mode depending on the variable `self.dark_mode`
		ctx.set_visuals(match self.dark_mode {
			true => Visuals::dark(),
			false => Visuals::light(),
		});

		// if text boxes aren't in focus, allow H keybind to toggle side panel.
		// this is behind this check as if it wasn't, it would trigger if the user
		// presses the h key in a text box as well
		if !self.text_boxes_focused {
			self.get_opened_mut("side_panel")
				.bitxor_assign(ctx.input().key_down(Key::H));
		}

		// Initialize fonts
		ctx.set_fonts(ASSETS.fonts.clone());

		// Creates Top bar that contains some general options
		TopBottomPanel::top("top_bar").show(ctx, |ui| {
			ui.horizontal(|ui| {
				// Button in top bar to toggle showing the side panel
				let side_curr_open = self.get_opened("help");
				self.get_opened_mut("side_panel").bitxor_assign(
					ui.add(Button::new("Panel"))
						.on_hover_text(match side_curr_open {
							true => "Hide Side Panel",
							false => "Show Side Panel",
						})
						.clicked(),
				);

				// Button to add a new function
				if ui
					.add(Button::new("Add Function"))
					.on_hover_text("Create and graph new function")
					.clicked()
				{
					self.functions.push(DEFAULT_FUNCTION_ENTRY.clone());
					self.func_strs.push(String::new());
					self.func_errors.push(None);
				}

				// Toggles opening the Help window
				let help_curr_open = self.get_opened("help");
				self.get_opened_mut("help").bitxor_assign(
					ui.add(Button::new("Help"))
						.on_hover_text(match help_curr_open {
							true => "Close Help Window",
							false => "Open Help Window",
						})
						.clicked(),
				);

				// Toggles opening the Info window
				let info_curr_open = self.get_opened("info");
				self.get_opened_mut("info").bitxor_assign(
					ui.add(Button::new("Info"))
						.on_hover_text(match info_curr_open {
							true => "Close Info Window",
							false => "Open Info Window",
						})
						.clicked(),
				);

				// Toggles dark/light mode
				self.dark_mode.bitxor_assign(
					ui.add(Button::new(match self.dark_mode {
						true => "🌞",
						false => "🌙",
					}))
					.on_hover_text(match self.dark_mode {
						true => "Turn the Lights on!",
						false => "Turn the Lights off.",
					})
					.clicked(),
				);

				// Display Area and time of last frame
				ui.label(format!(
					"Area: {} Took: {:?}",
					option_vec_printer(&self.last_info.0),
					self.last_info.1
				));
			});
		});

		// Help window with information for users
		Window::new("Help")
			.default_pos([200.0, 200.0])
			.open(self.get_opened_mut("help"))
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.heading("Help With...");

				ui.collapsing("Supported Expressions", |ui| {
					ui.label(&ASSETS.text_help_expr);
				});

				ui.collapsing("Supported Constants", |ui| {
					ui.label(&ASSETS.text_help_vars);
				});

				ui.collapsing("Panel", |ui| {
					ui.label(&ASSETS.text_help_panel);
				});

				ui.collapsing("Functions", |ui| {
					ui.label(&ASSETS.text_help_function);
				});

				ui.collapsing("Other", |ui| {
					ui.label(&ASSETS.text_help_other);
				});
			});

		// Window with information about the build and current commit
		Window::new("Info")
			.default_pos([200.0, 200.0])
			.open(self.get_opened_mut("info"))
			.resizable(false)
			.collapsible(false)
			.show(ctx, |ui| {
				ui.label(&*BUILD_INFO);
			});

		// If side panel is enabled, show it.
		if self.get_opened("side_panel") {
			self.side_panel(ctx);
		} else {
			self.text_boxes_focused = false;
		}

		// Referenced in plotting code, but needs to be here so it can be later
		// referenced when storing `last_info`
		let mut area_list: Vec<Option<f64>> = Vec::new();

		// Central panel which contains the central plot (or an error created when
		// parsing)
		CentralPanel::default().show(ctx, |ui| {
			// Display an error if it exists
			if self.exists_error {
				ui.centered_and_justified(|ui| {
					self.func_errors
						.iter()
						.filter(|ele| ele.is_some())
						.map(|ele| ele.as_ref().unwrap())
						.for_each(|ele| {
							ui.heading(format!("(Function #{}) {}\n", ele.0, ele.1));
						})
				});
				return;
			}

			let available_width: usize = (ui.available_width() as usize) + 1; // Used in later logic
			let width_changed = available_width != self.settings.plot_width;

			if width_changed {
				self.settings.plot_width = available_width;
			}

			// Create and setup plot
			Plot::new("plot")
				.set_margin_fraction(Vec2::ZERO)
				.data_aspect(1.0)
				.include_y(0)
				.legend(egui::plot::Legend::default())
				.show(ui, |plot_ui| {
					let bounds = plot_ui.plot_bounds();
					let minx_bounds: f64 = bounds.min()[0];
					let maxx_bounds: f64 = bounds.max()[0];

					dyn_mut_iter(&mut self.functions)
						.enumerate()
						.for_each(|(_, function)| {
							function.calculate(
								&minx_bounds,
								&maxx_bounds,
								width_changed,
								&self.settings,
							)
						});

					area_list = self
						.functions
						.iter()
						.enumerate()
						.map(|(_, function)| function.display(plot_ui, &self.settings))
						.collect();
				});
		});
		// Store list of functions' areas along with the time it took to process.
		self.last_info = (area_list, start.elapsed());
	}
}
