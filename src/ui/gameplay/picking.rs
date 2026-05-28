use super::*;

impl GameplayState {
    pub(crate) fn colonist_id_at_mouse(&self) -> Option<u32> {
        let game_area = self.layout.game_area();
        let mouse = mouse_position_vec2();
        let mouse_x = mouse.x;
        let mouse_y = mouse.y;
        if mouse_x < game_area.x
            || mouse_x > game_area.x + game_area.w
            || mouse_y < game_area.y
            || mouse_y > game_area.y + game_area.h
        {
            return None;
        }

        self.data
            .colonists
            .iter()
            .filter(|colonist| !colonist.is_on_mission())
            .filter_map(|colonist| {
                let foot = self.iso_view().grid_to_screen(colonist.position);
                let center_x = foot.x;
                let center_y = foot.y - 8.0;
                let dx = mouse_x - center_x;
                let dy = mouse_y - center_y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= 18.0 * 18.0 {
                    Some((colonist.id, distance_sq))
                } else {
                    None
                }
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(id, _)| id)
    }

    pub(crate) fn building_at_mouse(&self) -> Option<&Building> {
        let game_area = self.layout.game_area();
        let mouse = mouse_position_vec2();
        if !game_area.contains(mouse) {
            return None;
        }

        let grid_pos = self.iso_view().screen_to_grid(mouse);
        self.data.building_system.get_building_at(grid_pos)
    }

    pub(crate) fn colonist_by_id(&self, id: u32) -> Option<&Colonist> {
        self.data
            .colonists
            .iter()
            .find(|colonist| colonist.id == id)
    }

    pub(crate) fn inspected_colonist(&self, hovered_colonist_id: Option<u32>) -> Option<&Colonist> {
        hovered_colonist_id
            .and_then(|id| self.colonist_by_id(id))
            .or_else(|| {
                self.selected_colonist_id
                    .and_then(|id| self.colonist_by_id(id))
            })
    }
}
