//! Segmentation Table
//! 
//! Look up table for segmentation objects. Holds a set of objects and provides translatations 
//! between object name, index and display color. Currently used to generate segmentation
//! materials for 'bevy_image_segmentation'.
//! 
//! 
//! 

use bevy::{
    color::Color,
    ecs::prelude::Resource
};

use crate::utils::random_color;

/// SegmentationDataTable
/// Stores all object names and can generate a unique display color for each. 
#[derive(Resource)]
pub struct SegmentationDataTable {
    class_labels: Vec<String>,
    class_colors: Vec<Color>
}

impl SegmentationDataTable {
    pub fn index_of_label(&self, label: &String) -> Option<usize> {
        self.class_labels.iter().position(|n| n == label)
    }

    pub fn index_of_color(&self, color: &Color) -> Option<usize> {
        self.class_colors.iter().position(|c| c == color)
    }

    pub fn new_color(&self) -> Color {
        let mut new_color = random_color();
        while self.index_of_color(&new_color) != None {
            new_color = random_color();
        };
        new_color
    }

    pub fn label_id(&mut self, label: String) -> usize {
        match self.index_of_label(&label) {
            Some(id) => id,
            None => {
                self.class_labels.push(label);
                self.class_colors.push(self.new_color());
                self.class_labels.len() - 1
            }
        }
    }

    pub fn color_of_object_assertive(&mut self, label: String) -> Color {
        let index = self.label_id(label);
        self.class_colors[index].clone()
    }

    // fn color_id(&mut self, color: Color) -> Option<usize> {
    //     self.index_of_color(&color)
    // }

    // fn label_of(&self, index: usize) -> String {
    //     if index > self.class_labels.len() {return self.class_labels[0].clone()}
    //     self.class_labels[index].clone()
    // }

    // fn color_of(&self, index: usize) -> Color {
    //     if index > self.class_labels.len() {return self.class_colors[0].clone()}
    //     self.class_colors[index].clone()
    // }

    // fn color_of_object(&self, label: &String) -> Color {
    //     let index = match self.index_of_label(label) {
    //         Some(id) => id,
    //         None => 0,
    //     };
    //     self.class_colors[index].clone()
    // }

}

impl Default for SegmentationDataTable {
    fn default() -> Self {
        SegmentationDataTable {
            class_labels: vec![String::from("other")],
            class_colors: vec![Color::srgba(0.0, 0.0, 0.0, 0.0)]
        }
    }
}