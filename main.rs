use eframe::egui;

mod gstructs;
mod memory;

fn main() {
    let mut options = eframe::NativeOptions::default();
    let window_size = Some(eframe::egui::Vec2{x: 700.0, y: 300.0}); 
    options.initial_window_size = window_size;
    eframe::run_native(
        "Injector",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
   
}

struct MyApp {
    proc_list: Vec<String>,
    process_selected: String,
    injection_method: String
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            proc_list: memory::get_process_list(),
            process_selected: String::from(""),
            injection_method: String::from("Manual Map")
        }
    }
    
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DLL Injector");
            ui.label("Select a process to inject the DLL");
            ui.end_row();
            egui::ComboBox::from_label("")
                .width(500.0)
                .selected_text(self.process_selected.to_string())
                .show_ui(ui, |ui| {
                    for process in &self.proc_list { // TODO remove duplictes / add PID to differentiate between dupes
                        ui.selectable_value(&mut self.process_selected, process.to_string(), process.to_string());
                    }
                }
            );
            ui.end_row();
            ui.label("Select a DLL to inject");
            ui.end_row();
            egui::ComboBox::from_id_source("#8")
                .width(500.0)
                .selected_text(self.injection_method.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.injection_method, String::from("Manual Map"), "Manual Map");
                    ui.selectable_value(&mut self.injection_method, String::from("LoadLibrary"), "LoadLibrary");
                    ui.selectable_value(&mut self.injection_method, String::from("LdrLoadDLL"), "LdrLoadDLL");
                }
            );
            ui.end_row();
            if ui.button("Inject DLL").clicked() {
                //
            }
        });
    }
}