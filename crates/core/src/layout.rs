use freya_engine::prelude::*;
use freya_native_core::{
    prelude::NodeImmutable,
    NodeId,
};
use itertools::Itertools;
use torin::geometry::Area;

use crate::{
    dom::*,
    prelude::{
        ElementUtils,
        ElementUtilsResolver,
    },
    skia::SkiaMeasurer,
};

/// Process the layout of the DOM
pub fn process_layout(
    fdom: &FreyaDOM,
    area: Area,
    font_collection: &mut FontCollection,
    scale_factor: f64,
    default_fonts: &[String],
) {
    {
        let rdom = fdom.rdom();
        let mut dom_adapter = DioxusDOMAdapter::new(rdom, scale_factor as f32);
        let skia_measurer =
            SkiaMeasurer::new(rdom, font_collection, default_fonts, scale_factor as f32);

        let mut layout = fdom.layout();

        // Finds the best Node from where to start measuring
        layout.find_best_root(&mut dom_adapter);

        let get_drawing_area = |node_id: &NodeId| -> Option<Area> {
            let layout_node = layout.get(*node_id)?;
            let node = rdom.get(*node_id)?;
            let utils = node.node_type().tag()?.utils()?;

            Some(utils.drawing_area(layout_node.visible_area(), &node, 1.0f32)) // TODO scale factor
        };

        // Unite the areas of the invalidated nodes with the dirty area
        let mut compositor_dirty_area = fdom.compositor_dirty_area();
        let mut buffer = layout.dirty.iter().copied().collect_vec();
        while let Some(node_id) = buffer.pop() {
            if let Some(area) = get_drawing_area(&node_id) {
                // Unite the invalidated area with the dirty area
                compositor_dirty_area.unite_or_insert(&area);

                if let Some(node) = rdom.get(node_id) {
                    buffer.extend(node.child_ids());
                }
            }
        }
        let root_id = fdom.rdom().root_id();

        // Measure the layout
        layout.measure(root_id, area, &mut Some(skia_measurer), &mut dom_adapter);
    }
}
