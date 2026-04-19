use anyhow::Result;
use uuid::Uuid;
use crate::project::Project;
use crate::project::clip::{TextStroke, TextShadow, RgbaColor, TextAlign, TextPositionPreset};
use crate::tools::Tool;

/// Actualiza el estilo de un clip de texto
pub struct SetTextStyle {
    pub track_id:   Uuid,
    pub clip_id:    Uuid,
    pub font_family:    Option<String>,
    pub font_size:      Option<u32>,
    pub color:          Option<RgbaColor>,
    pub bg_color:       Option<Option<RgbaColor>>,
    pub bold:           Option<bool>,
    pub italic:         Option<bool>,
    pub underline:      Option<bool>,
    pub align:          Option<TextAlign>,
    pub line_height:    Option<f64>,
    pub letter_spacing: Option<f64>,
    pub stroke_width:   Option<f64>,
    pub stroke_color:   Option<RgbaColor>,
    pub shadow:         Option<Option<TextShadow>>,
    // Posicionamiento
    pub position_preset: Option<TextPositionPreset>,
    pub pos_x:           Option<f64>,
    pub pos_y:           Option<f64>,
    pub margin:          Option<f64>,
    pub rotation_deg:    Option<f64>,
}

impl Tool for SetTextStyle {
    fn name(&self) -> &str { "set_text_style" }

    fn apply(&self, project: &mut Project) -> Result<()> {
        let track = project
            .track_mut(self.track_id)
            .ok_or_else(|| anyhow::anyhow!("Track {} no encontrado", self.track_id))?;
        let clip = track
            .text_clip_mut(self.clip_id)
            .ok_or_else(|| anyhow::anyhow!("TextClip {} no encontrado", self.clip_id))?;

        if let Some(ref f) = self.font_family    { clip.style.font_family    = f.clone(); }
        if let Some(s)    = self.font_size        { clip.style.font_size      = s; }
        if let Some(ref c) = self.color           { clip.style.color          = c.clone(); }
        if let Some(ref bg) = self.bg_color       { clip.style.bg_color       = bg.clone(); }
        if let Some(b)    = self.bold             { clip.style.bold           = b; }
        if let Some(i)    = self.italic           { clip.style.italic         = i; }
        if let Some(u)    = self.underline        { clip.style.underline      = u; }
        if let Some(ref a) = self.align           { clip.style.align          = a.clone(); }
        if let Some(lh)   = self.line_height      { clip.style.line_height    = lh.max(0.5); }
        if let Some(ls)   = self.letter_spacing   { clip.style.letter_spacing = ls; }

        if let Some(sw) = self.stroke_width {
            let color = self.stroke_color.clone().unwrap_or(RgbaColor::black());
            if sw > 0.0 {
                clip.style.stroke = Some(TextStroke { width: sw, color });
            } else {
                clip.style.stroke = None;
            }
        }
        if let Some(ref sh) = self.shadow         { clip.style.shadow         = sh.clone(); }

        // Posicionamiento
        if let Some(ref p) = self.position_preset { clip.position_preset = p.clone(); }
        if let Some(x)    = self.pos_x            { clip.pos_x = Some(x); }
        if let Some(y)    = self.pos_y            { clip.pos_y = Some(y); }
        if let Some(m)    = self.margin           { clip.margin = m.max(0.0); }
        if let Some(r)    = self.rotation_deg     { clip.rotation_deg = r; }

        tracing::info!("Estilo actualizado en TextClip {}", self.clip_id);
        Ok(())
    }
}
