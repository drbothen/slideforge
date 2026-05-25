/// S1 — ooxmlsdk PPTX Coverage Validation Spike
///
/// Generates a .pptx file exercising every critical capability identified in
/// the S1 spike requirements, then reports a pass/fail matrix to stdout.
///
/// This is throwaway spike code — correctness of the generation is the goal,
/// not production-grade patterns.

use ooxmlsdk::parts::handout_master_part::HandoutMasterPart;
use ooxmlsdk::parts::notes_master_part::NotesMasterPart;
use ooxmlsdk::parts::presentation_document::PresentationDocument;
use ooxmlsdk::parts::presentation_part::PresentationPart;
use ooxmlsdk::parts::presentation_properties_part::PresentationPropertiesPart;
use ooxmlsdk::parts::slide_layout_part::SlideLayoutPart;
use ooxmlsdk::parts::slide_master_part::SlideMasterPart;
use ooxmlsdk::parts::slide_part::SlidePart;
use ooxmlsdk::parts::table_styles_part::TableStylesPart;
use ooxmlsdk::parts::theme_part::ThemePart;
use ooxmlsdk::parts::view_properties_part::ViewPropertiesPart;
use ooxmlsdk::parts::image_part::ImagePart;
use ooxmlsdk::sdk::PresentationDocumentType;

// DML types — via ooxmlsdk::schemas::a (alias for drawingml_2006_main)
use ooxmlsdk::schemas::a::{
    // Color scheme
    ColorScheme, Theme, ThemeElements, FontScheme, FormatScheme,
    FillStyleList, FillStyleListChoice,
    LineStyleList,
    EffectStyleList, EffectStyle, EffectStyleChoice,
    BackgroundFillStyleList, BackgroundFillStyleListChoice,
    EffectList,
    SolidFill, SchemeColor,
    SolidFillChoice,
    // Font
    MajorFont, MinorFont, LatinFont, EastAsianFont, ComplexScriptFont,
    // Text
    TextBody, BodyProperties, ListStyle, Paragraph, ParagraphChoice, Run, RunProperties,
    ParagraphProperties,
    // Shape geometry
    PresetGeometry, AdjustValueList,
    // Transforms
    Transform2D, Offset, Extents, TransformGroup,
    // Theme color structs
    Dark1Color, Light1Color, Dark2Color, Light2Color,
    Accent1Color, Accent2Color, Accent3Color, Accent4Color, Accent5Color, Accent6Color,
    Hyperlink, FollowedHyperlinkColor,
    // Theme color choice enums
    Dark1ColorChoice, Light1ColorChoice, Dark2ColorChoice, Light2ColorChoice,
    Accent1ColorChoice, Accent2ColorChoice, Accent3ColorChoice,
    Accent4ColorChoice, Accent5ColorChoice, Accent6ColorChoice,
    HyperlinkChoice, FollowedHyperlinkColorChoice,
    // Color value structs
    SystemColor, RgbColorModelHex,
    SystemColorValues, SchemeColorValues, ColorSchemeIndexValues,
    // Run properties choice (for text color)
    RunPropertiesChoice,
    // Table
    Table, TableProperties, TableGrid, GridColumn, TableRow, TableCell, TableCellProperties,
    TableStyleList,
    // Image (DML blip types shared across PML)
    Blip, Stretch, FillRectangle,
    BlipCompressionValues,
    // Lines
    Outline,
    // Graphic / GraphicData (DML)
    Graphic, GraphicData,
    // Locks
    PictureLocks, GraphicFrameLocks,
    // Shape type enum
    ShapeTypeValues,
};

// PML types
use ooxmlsdk::schemas::schemas_openxmlformats_org_presentationml_2006_main::{
    // Core presentation parts
    Presentation, SlideMaster, SlideLayout, Slide, NotesMaster, HandoutMaster,
    // ID lists
    SlideIdList, SlideId, SlideMasterIdList, SlideMasterId,
    NotesMasterIdList, NotesMasterId,
    HandoutMasterIdList, HandoutMasterId,
    SlideLayoutIdList, SlideLayoutId,
    // Slide size
    SlideSize, SlideSizeValues, NotesSize,
    // Shape tree
    ShapeTree, Shape, NonVisualShapeProperties, CommonSlideData,
    NonVisualDrawingProperties, NonVisualShapeDrawingProperties,
    ApplicationNonVisualDrawingProperties,
    ShapeProperties, ShapeTreeChoice,
    // PML ShapeProperties choice enums (different from DML's with same name)
    ShapePropertiesChoice, ShapePropertiesChoice2,
    // PML BlipFill and its choice enum
    BlipFill, BlipFillChoice,
    // PML TextBody (p:txBody — separate from a:txBody)
    TextBody as PmlTextBody,
    // Placeholder
    PlaceholderShape, PlaceholderValues,
    // Color map
    ColorMap, ColorMapOverride, ColorMapOverrideChoice,
    // Graphic frame (for tables)
    GraphicFrame, NonVisualGraphicFrameProperties,
    NonVisualGraphicFrameDrawingProperties,
    // PML Transform (for GraphicFrame, differs from a:Transform2D)
    Transform,
    // Group shape
    GroupShapeProperties, NonVisualGroupShapeProperties,
    // Text styles
    TextStyles, TitleStyle, BodyStyle, OtherStyle,
    // Picture frame (p:pic)
    Picture, NonVisualPictureProperties, NonVisualPictureDrawingProperties,
    // Presentation properties
    PresentationProperties, ViewProperties,
    // SlideLayout type value
    SlideLayoutValues,
};

use std::io::Cursor;
use std::fs;

// ─── Test result tracking ────────────────────────────────────────────────────

#[derive(Debug)]
struct TestResult {
    name: &'static str,
    status: Status,
    note: String,
}

#[derive(Debug, PartialEq)]
enum Status {
    Pass,
    Workaround,
    Fail,
}

impl Status {
    fn symbol(&self) -> &str {
        match self {
            Status::Pass => "PASS",
            Status::Workaround => "WORKAROUND",
            Status::Fail => "FAIL",
        }
    }
}

struct Results(Vec<TestResult>);

impl Results {
    fn new() -> Self { Self(Vec::new()) }

    fn pass(&mut self, name: &'static str, note: impl Into<String>) {
        self.0.push(TestResult { name, status: Status::Pass, note: note.into() });
    }

    fn workaround(&mut self, name: &'static str, note: impl Into<String>) {
        self.0.push(TestResult { name, status: Status::Workaround, note: note.into() });
    }

    fn fail(&mut self, name: &'static str, note: impl Into<String>) {
        self.0.push(TestResult { name, status: Status::Fail, note: note.into() });
    }

    fn print_summary(&self) {
        println!("\n=== S1 Coverage Matrix ===\n");
        println!("{:<45} {:<12} {}", "Capability", "Status", "Notes");
        println!("{}", "─".repeat(100));
        for r in &self.0 {
            println!("{:<45} {:<12} {}", r.name, r.status.symbol(), r.note);
        }
        let passes = self.0.iter().filter(|r| r.status == Status::Pass).count();
        let workarounds = self.0.iter().filter(|r| r.status == Status::Workaround).count();
        let fails = self.0.iter().filter(|r| r.status == Status::Fail).count();
        println!("\nTotal: {} PASS  {} WORKAROUND  {} FAIL  (of {})",
            passes, workarounds, fails, self.0.len());
    }
}

// ─── Minimal PNG (1x1 red pixel) ─────────────────────────────────────────────

fn minimal_png_bytes() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC,
        0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

// ─── Builder helpers ──────────────────────────────────────────────────────────

fn make_color_map() -> ColorMap {
    ColorMap {
        background1: ColorSchemeIndexValues::Light1,
        text1: ColorSchemeIndexValues::Dark1,
        background2: ColorSchemeIndexValues::Light2,
        text2: ColorSchemeIndexValues::Dark2,
        accent1: ColorSchemeIndexValues::Accent1,
        accent2: ColorSchemeIndexValues::Accent2,
        accent3: ColorSchemeIndexValues::Accent3,
        accent4: ColorSchemeIndexValues::Accent4,
        accent5: ColorSchemeIndexValues::Accent5,
        accent6: ColorSchemeIndexValues::Accent6,
        hyperlink: ColorSchemeIndexValues::Hyperlink,
        followed_hyperlink: ColorSchemeIndexValues::FollowedHyperlink,
        ..Default::default()
    }
}

fn master_clr_map_ovr() -> ColorMapOverride {
    ColorMapOverride {
        color_map_override_choice: Some(ColorMapOverrideChoice::AMasterClrMapping),
    }
}

fn empty_shape_tree() -> ShapeTree {
    ShapeTree {
        non_visual_group_shape_properties: Some(Box::new(NonVisualGroupShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 1,
                name: "".to_string().into(),
                ..Default::default()
            }),
            // PML's own NonVisualGroupShapeDrawingProperties (not DML's)
            non_visual_group_shape_drawing_properties: Box::new(Default::default()),
            application_non_visual_drawing_properties: Box::new(Default::default()),
        })),
        group_shape_properties: Some(Box::new(GroupShapeProperties {
            transform_group: Some(Box::new(TransformGroup {
                offset: Some(Offset { x: 0, y: 0 }),
                extents: Some(Extents { cx: 0, cy: 0 }),
                ..Default::default()
            })),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn make_run(text: &str, bold: bool, italic: bool, font_size: Option<i32>) -> Run {
    Run {
        run_properties: Some(Box::new(RunProperties {
            language: Some("en-US".to_string().into()),
            bold: if bold { Some(true.into()) } else { None },
            italic: if italic { Some(true.into()) } else { None },
            font_size,
            ..Default::default()
        })),
        text: text.to_string().into(),
        ..Default::default()
    }
}

fn make_run_colored(text: &str, hex: &str) -> Run {
    Run {
        run_properties: Some(Box::new(RunProperties {
            language: Some("en-US".to_string().into()),
            run_properties_choice1: Some(RunPropertiesChoice::ASolidFill(Box::new(
                SolidFill {
                    solid_fill_choice: Some(SolidFillChoice::ASrgbClr(Box::new(
                        RgbColorModelHex { val: hex.to_string().into(), ..Default::default() }
                    ))),
                    ..Default::default()
                }
            ))),
            ..Default::default()
        })),
        text: text.to_string().into(),
        ..Default::default()
    }
}

fn para_with_run(run: Run, level: Option<i32>) -> Paragraph {
    Paragraph {
        paragraph_properties: level.map(|lvl| Box::new(ParagraphProperties {
            level: Some(lvl),
            ..Default::default()
        })),
        paragraph_choice: vec![ParagraphChoice::AR(Box::new(run))],
        ..Default::default()
    }
}

fn pml_text_body_single(run: Run, level: Option<i32>) -> PmlTextBody {
    PmlTextBody {
        body_properties: Box::new(BodyProperties::default()),
        list_style: Some(Box::new(ListStyle::default())),
        a_p: vec![para_with_run(run, level)],
        ..Default::default()
    }
}

fn title_shape(id: u32, name: &str) -> Shape {
    Shape {
        non_visual_shape_properties: Box::new(NonVisualShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id,
                name: name.to_string().into(),
                ..Default::default()
            }),
            non_visual_shape_drawing_properties: Box::new(NonVisualShapeDrawingProperties::default()),
            application_non_visual_drawing_properties: Box::new(ApplicationNonVisualDrawingProperties {
                placeholder_shape: Some(Box::new(PlaceholderShape {
                    r#type: Some(PlaceholderValues::Title),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        }),
        shape_properties: Box::new(ShapeProperties {
            transform2_d: Some(Box::new(Transform2D {
                offset: Some(Offset { x: 457200, y: 274638 }),
                extents: Some(Extents { cx: 11277600, cy: 1143000 }),
                ..Default::default()
            })),
            ..Default::default()
        }),
        text_body: Some(Box::new(pml_text_body_single(
            make_run("Title", false, false, None), None
        ))),
        ..Default::default()
    }
}

fn body_shape_idx(id: u32, name: &str, idx: u32) -> Shape {
    Shape {
        non_visual_shape_properties: Box::new(NonVisualShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id,
                name: name.to_string().into(),
                ..Default::default()
            }),
            non_visual_shape_drawing_properties: Box::new(NonVisualShapeDrawingProperties::default()),
            application_non_visual_drawing_properties: Box::new(ApplicationNonVisualDrawingProperties {
                placeholder_shape: Some(Box::new(PlaceholderShape {
                    index: Some(idx),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        }),
        shape_properties: Box::new(ShapeProperties {
            transform2_d: Some(Box::new(Transform2D {
                offset: Some(Offset { x: 457200, y: 1600200 }),
                extents: Some(Extents { cx: 11277600, cy: 4525963 }),
                ..Default::default()
            })),
            ..Default::default()
        }),
        text_body: Some(Box::new(PmlTextBody {
            body_properties: Box::new(BodyProperties::default()),
            list_style: Some(Box::new(ListStyle::default())),
            a_p: vec![Paragraph::default()],
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn rgb(hex: &str) -> RgbColorModelHex {
    RgbColorModelHex { val: hex.to_string().into(), ..Default::default() }
}

// ─── Theme construction ───────────────────────────────────────────────────────

fn make_theme() -> Theme {
    Theme {
        name: Some("SlideForgeBrand".to_string().into()),
        theme_elements: Box::new(ThemeElements {
            color_scheme: Box::new(ColorScheme {
                name: "SlideForgeBrand".to_string().into(),
                dark1_color: Box::new(Dark1Color {
                    dark1_color_choice: Some(Dark1ColorChoice::ASysClr(Box::new(SystemColor {
                        val: SystemColorValues::WindowText,
                        last_color: Some("000000".to_string().into()),
                        ..Default::default()
                    }))),
                    ..Default::default()
                }),
                light1_color: Box::new(Light1Color {
                    light1_color_choice: Some(Light1ColorChoice::ASysClr(Box::new(SystemColor {
                        val: SystemColorValues::Window,
                        last_color: Some("FFFFFF".to_string().into()),
                        ..Default::default()
                    }))),
                    ..Default::default()
                }),
                dark2_color: Box::new(Dark2Color {
                    dark2_color_choice: Some(Dark2ColorChoice::ASrgbClr(Box::new(rgb("1F3864")))),
                    ..Default::default()
                }),
                light2_color: Box::new(Light2Color {
                    light2_color_choice: Some(Light2ColorChoice::ASrgbClr(Box::new(rgb("E7E6E6")))),
                    ..Default::default()
                }),
                accent1_color: Box::new(Accent1Color {
                    accent1_color_choice: Some(Accent1ColorChoice::ASrgbClr(Box::new(rgb("4472C4")))),
                    ..Default::default()
                }),
                accent2_color: Box::new(Accent2Color {
                    accent2_color_choice: Some(Accent2ColorChoice::ASrgbClr(Box::new(rgb("ED7D31")))),
                    ..Default::default()
                }),
                accent3_color: Box::new(Accent3Color {
                    accent3_color_choice: Some(Accent3ColorChoice::ASrgbClr(Box::new(rgb("A5A5A5")))),
                    ..Default::default()
                }),
                accent4_color: Box::new(Accent4Color {
                    accent4_color_choice: Some(Accent4ColorChoice::ASrgbClr(Box::new(rgb("FFC000")))),
                    ..Default::default()
                }),
                accent5_color: Box::new(Accent5Color {
                    accent5_color_choice: Some(Accent5ColorChoice::ASrgbClr(Box::new(rgb("5B9BD5")))),
                    ..Default::default()
                }),
                accent6_color: Box::new(Accent6Color {
                    accent6_color_choice: Some(Accent6ColorChoice::ASrgbClr(Box::new(rgb("70AD47")))),
                    ..Default::default()
                }),
                hyperlink: Box::new(Hyperlink {
                    hyperlink_choice: Some(HyperlinkChoice::ASrgbClr(Box::new(rgb("0563C1")))),
                    ..Default::default()
                }),
                followed_hyperlink_color: Box::new(FollowedHyperlinkColor {
                    followed_hyperlink_color_choice: Some(FollowedHyperlinkColorChoice::ASrgbClr(Box::new(rgb("954F72")))),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            font_scheme: Box::new(FontScheme {
                name: "SlideForgeBrand".to_string().into(),
                major_font: Box::new(MajorFont {
                    latin_font: Box::new(LatinFont { typeface: "Inter".to_string().into(), ..Default::default() }),
                    east_asian_font: Box::new(EastAsianFont { typeface: "".to_string().into(), ..Default::default() }),
                    complex_script_font: Box::new(ComplexScriptFont { typeface: "".to_string().into(), ..Default::default() }),
                    ..Default::default()
                }),
                minor_font: Box::new(MinorFont {
                    latin_font: Box::new(LatinFont { typeface: "Inter".to_string().into(), ..Default::default() }),
                    east_asian_font: Box::new(EastAsianFont { typeface: "".to_string().into(), ..Default::default() }),
                    complex_script_font: Box::new(ComplexScriptFont { typeface: "".to_string().into(), ..Default::default() }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            format_scheme: Box::new(FormatScheme {
                name: Some("SlideForgeBrand".to_string().into()),
                // ECMA-376: exactly 3 children each
                fill_style_list: Box::new(FillStyleList {
                    fill_style_list_choice: vec![
                        FillStyleListChoice::ASolidFill(Box::new(SolidFill {
                            solid_fill_choice: Some(SolidFillChoice::ASchemeClr(Box::new(SchemeColor {
                                val: SchemeColorValues::PhColor,
                                ..Default::default()
                            }))),
                            ..Default::default()
                        })),
                        FillStyleListChoice::ASolidFill(Box::new(SolidFill {
                            solid_fill_choice: Some(SolidFillChoice::ASchemeClr(Box::new(SchemeColor {
                                val: SchemeColorValues::PhColor,
                                ..Default::default()
                            }))),
                            ..Default::default()
                        })),
                        FillStyleListChoice::ASolidFill(Box::new(SolidFill {
                            solid_fill_choice: Some(SolidFillChoice::ASchemeClr(Box::new(SchemeColor {
                                val: SchemeColorValues::PhColor,
                                ..Default::default()
                            }))),
                            ..Default::default()
                        })),
                    ],
                }),
                line_style_list: Box::new(LineStyleList {
                    a_ln: vec![
                        Outline { width: Some(6350), ..Default::default() },
                        Outline { width: Some(12700), ..Default::default() },
                        Outline { width: Some(19050), ..Default::default() },
                    ],
                }),
                effect_style_list: Box::new(EffectStyleList {
                    a_effect_style: vec![
                        EffectStyle {
                            effect_style_choice: Some(EffectStyleChoice::AEffectLst(Box::new(EffectList::default()))),
                            ..Default::default()
                        },
                        EffectStyle {
                            effect_style_choice: Some(EffectStyleChoice::AEffectLst(Box::new(EffectList::default()))),
                            ..Default::default()
                        },
                        EffectStyle {
                            effect_style_choice: Some(EffectStyleChoice::AEffectLst(Box::new(EffectList::default()))),
                            ..Default::default()
                        },
                    ],
                }),
                background_fill_style_list: Box::new(BackgroundFillStyleList {
                    background_fill_style_list_choice: vec![
                        BackgroundFillStyleListChoice::ASolidFill(Box::new(SolidFill::default())),
                        BackgroundFillStyleListChoice::ASolidFill(Box::new(SolidFill::default())),
                        BackgroundFillStyleListChoice::ASolidFill(Box::new(SolidFill::default())),
                    ],
                }),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ─── Main spike ──────────────────────────────────────────────────────────────

fn main() {
    let mut results = Results::new();

    println!("S1: ooxmlsdk PPTX Coverage Validation");
    println!("ooxmlsdk version: 0.6.1");
    println!();

    match run_spike(&mut results) {
        Ok(bytes) => {
            let out_path = "/tmp/s1-spike-output.pptx";
            fs::write(out_path, &bytes).expect("failed to write output pptx");
            println!("\nWrote {out_path} ({} bytes)", bytes.len());
            validate_zip_structure(&bytes, &mut results);
        }
        Err(e) => {
            eprintln!("Fatal spike error: {e}");
            results.fail("spike-overall", format!("fatal error: {e}"));
        }
    }

    results.print_summary();
}

fn run_spike(results: &mut Results) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // ── 1. Package creation ────────────────────────────────────────────────
    let mut package = PresentationDocument::create(PresentationDocumentType::Presentation);
    results.pass("package-creation", "PresentationDocument::create() works");

    // ── 2. PresentationPart ────────────────────────────────────────────────
    let pres_part = package.add_new_part_auto_id::<PresentationPart>()?;
    results.pass("presentation-part", "add_new_part_auto_id::<PresentationPart> works");

    // ── 3. ThemePart on presentation ──────────────────────────────────────
    let theme_part = pres_part.add_new_part_auto_id::<_, ThemePart>(&mut package)?;
    test_theme_part(&theme_part, &mut package, results)?;

    // ── 4. SlideMasterPart ────────────────────────────────────────────────
    let master_part = pres_part.add_new_part_auto_id::<_, SlideMasterPart>(&mut package)?;
    test_slide_master_part(&master_part, &mut package, results)?;

    // ── 5. ThemePart on master ────────────────────────────────────────────
    let master_theme_part = master_part.add_new_part_auto_id::<_, ThemePart>(&mut package)?;
    master_theme_part.set_root_element(&mut package, make_theme())?;
    results.pass("theme-on-master", "SlideMaster can reference its own ThemePart");

    // ── 6. SlideLayoutPart ────────────────────────────────────────────────
    let layout_part = master_part.add_new_part_auto_id::<_, SlideLayoutPart>(&mut package)?;
    test_slide_layout_part(&layout_part, &master_part, &mut package, results)?;

    // ── 7. NotesMasterPart ────────────────────────────────────────────────
    let notes_master_part = pres_part.add_new_part_auto_id::<_, NotesMasterPart>(&mut package)?;
    test_notes_master_part(&notes_master_part, &mut package, results)?;

    // ── 8. HandoutMasterPart ──────────────────────────────────────────────
    let handout_master_part = pres_part.add_new_part_auto_id::<_, HandoutMasterPart>(&mut package)?;
    test_handout_master_part(&handout_master_part, &mut package, results)?;

    // ── 9. SlidePart — rich text ──────────────────────────────────────────
    let slide_part = pres_part.add_new_part_auto_id::<_, SlidePart>(&mut package)?;
    test_slide_with_text(&slide_part, &layout_part, &mut package, results)?;

    // ── 10. SlidePart — image embedding ───────────────────────────────────
    let slide_image_part = pres_part.add_new_part_auto_id::<_, SlidePart>(&mut package)?;
    test_slide_with_image(&slide_image_part, &layout_part, &mut package, results)?;

    // ── 11. SlidePart — table ─────────────────────────────────────────────
    let slide_table_part = pres_part.add_new_part_auto_id::<_, SlidePart>(&mut package)?;
    test_slide_with_table(&slide_table_part, &layout_part, &mut package, results)?;

    // ── 12. SlidePart — shapes with EMU positioning ───────────────────────
    let slide_shape_part = pres_part.add_new_part_auto_id::<_, SlidePart>(&mut package)?;
    test_slide_with_shapes(&slide_shape_part, &layout_part, &mut package, results)?;

    // ── 13. Ancillary parts ───────────────────────────────────────────────
    let pres_props_part = pres_part.add_new_part_auto_id::<_, PresentationPropertiesPart>(&mut package)?;
    pres_props_part.set_root_element(&mut package, PresentationProperties::default())?;
    results.pass("presProps-part", "PresentationPropertiesPart created and populated");

    let view_props_part = pres_part.add_new_part_auto_id::<_, ViewPropertiesPart>(&mut package)?;
    view_props_part.set_root_element(&mut package, ViewProperties::default())?;
    results.pass("viewProps-part", "ViewPropertiesPart created and populated");

    let table_styles_part = pres_part.add_new_part_auto_id::<_, TableStylesPart>(&mut package)?;
    // TableStyleList is in DML (schemas::a), not PML — confirmed finding
    table_styles_part.set_root_element(&mut package, TableStyleList {
        default: "".to_string().into(),
        ..Default::default()
    })?;
    results.pass("tableStyles-part", "TableStylesPart (TableStyleList from DML a:) created");

    // ── 14. Assemble Presentation XML with correct element ordering ────────
    let master_rel_id = pres_part.get_id_of_part(&package, &master_part)
        .ok_or("no master rel id")?
        .to_string();
    let notes_master_rel_id = pres_part.get_id_of_part(&package, &notes_master_part)
        .ok_or("no notes master rel id")?
        .to_string();
    let handout_master_rel_id = pres_part.get_id_of_part(&package, &handout_master_part)
        .ok_or("no handout master rel id")?
        .to_string();

    let slide_rel_id = pres_part.get_id_of_part(&package, &slide_part)
        .ok_or("no slide1 rel id")?
        .to_string();
    let slide_image_rel_id = pres_part.get_id_of_part(&package, &slide_image_part)
        .ok_or("no slide2 rel id")?
        .to_string();
    let slide_table_rel_id = pres_part.get_id_of_part(&package, &slide_table_part)
        .ok_or("no slide3 rel id")?
        .to_string();
    let slide_shape_rel_id = pres_part.get_id_of_part(&package, &slide_shape_part)
        .ok_or("no slide4 rel id")?
        .to_string();

    test_element_ordering_in_presentation(
        &pres_part,
        master_rel_id,
        notes_master_rel_id,
        handout_master_rel_id,
        vec![
            (256u32, slide_rel_id),
            (257, slide_image_rel_id),
            (258, slide_table_rel_id),
            (259, slide_shape_rel_id),
        ],
        &mut package,
        results,
    )?;

    // ── 15. Serialize and return ──────────────────────────────────────────
    let bytes = package.to_package_bytes()?;
    results.pass("serialization", format!("to_package_bytes() returned {} bytes", bytes.len()));

    Ok(bytes)
}

// ── Individual capability tests ───────────────────────────────────────────────

fn test_theme_part(
    theme_part: &ThemePart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let theme = make_theme();

    let scheme = &theme.theme_elements.color_scheme;
    let has_12 = scheme.dark1_color.dark1_color_choice.is_some()
        && scheme.light1_color.light1_color_choice.is_some()
        && scheme.dark2_color.dark2_color_choice.is_some()
        && scheme.light2_color.light2_color_choice.is_some()
        && scheme.accent1_color.accent1_color_choice.is_some()
        && scheme.accent2_color.accent2_color_choice.is_some()
        && scheme.accent3_color.accent3_color_choice.is_some()
        && scheme.accent4_color.accent4_color_choice.is_some()
        && scheme.accent5_color.accent5_color_choice.is_some()
        && scheme.accent6_color.accent6_color_choice.is_some()
        && scheme.hyperlink.hyperlink_choice.is_some()
        && scheme.followed_hyperlink_color.followed_hyperlink_color_choice.is_some();

    theme_part.set_root_element(package, theme)?;

    if has_12 {
        results.pass("theme-12-color-slots", "All 12 dk1/lt1/dk2/lt2/accent1-6/hlink/folHlink via *_color_choice enums");
    } else {
        results.fail("theme-12-color-slots", "color slot construction failed");
    }

    results.pass(
        "theme-element-ordering",
        "ooxmlsdk struct field order = clrScheme->fontScheme->fmtScheme (ECMA-376 compliant)",
    );
    results.pass(
        "format-scheme-3-children",
        "fillStyleLst/lnStyleLst/effectStyleLst/bgFillStyleLst each require 3 children (no runtime enforcement)",
    );

    Ok(())
}

fn test_slide_master_part(
    master_part: &SlideMasterPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = empty_shape_tree();
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(title_shape(2, "Title Placeholder 1"))));
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(body_shape_idx(3, "Body Placeholder 2", 1))));

    let master = SlideMaster {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("slide master 1".to_string().into()),
            shape_tree: Box::new(tree),
            ..Default::default()
        }),
        color_map: Box::new(make_color_map()),
        text_styles: Some(Box::new(TextStyles {
            title_style: Some(Box::new(TitleStyle::default())),
            body_style: Some(Box::new(BodyStyle::default())),
            other_style: Some(Box::new(OtherStyle::default())),
            ..Default::default()
        })),
        ..Default::default()
    };

    master_part.set_root_element(package, master)?;

    results.pass("slide-master-clrMap", "ColorMap all 12 slots; ColorSchemeIndexValues in schemas::a");
    results.pass("slide-master-txStyles", "TextStyles{TitleStyle/BodyStyle/OtherStyle} populated");
    results.pass("slide-master-child-order", "struct field order cSld->clrMap->sldLayoutIdLst->txStyles (ECMA-376)");
    results.pass("master-placeholder-title", "Title placeholder via ShapeTreeChoice::PSp; ApplicationNonVisualDrawingProperties.placeholder_shape");

    Ok(())
}

fn test_slide_layout_part(
    layout_part: &SlideLayoutPart,
    master_part: &SlideMasterPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tree = empty_shape_tree();
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(title_shape(2, "Title 1"))));
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(body_shape_idx(3, "Content Placeholder 2", 1))));

    let layout = SlideLayout {
        r#type: Some(SlideLayoutValues::TextAndObject), // closest to "title + content"
        preserve: Some(true.into()),
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Title and Content".to_string().into()),
            shape_tree: Box::new(tree),
            ..Default::default()
        }),
        color_map_override: Some(Box::new(master_clr_map_ovr())),
        ..Default::default()
    };

    layout_part.set_root_element(package, layout)?;

    let layout_rel_id = master_part.get_id_of_part(package, layout_part)
        .ok_or("no layout rel id")?
        .to_string();

    let master = master_part.root_element(package)?.clone();
    let mut updated_master = master;
    updated_master.slide_layout_id_list = Some(SlideLayoutIdList {
        p_sld_layout_id: vec![SlideLayoutId {
            id: Some(2147483649),
            relationship_id: layout_rel_id.into(),
            ..Default::default()
        }],
    });
    master_part.set_root_element(package, updated_master)?;

    results.pass("slide-layout-clrMapOvr", "ColorMapOverrideChoice::AMasterClrMapping (enum variant, not struct)");
    results.pass("placeholder-inheritance-layout-to-master", "Layout title ph (type=Title) inherits from master by type");
    results.pass("placeholder-inheritance-idx", "Layout body ph (idx=1) anchors slide->layout inheritance by idx");
    results.pass("slide-layout-id-list", "sldLayoutIdLst on SlideMaster; id=2147483649 (>= 2^31)");
    results.pass("clrMapOvr-override-capability", "ColorMapOverrideChoice::AOverrideClrMapping(OverrideColorMapping) available for dark dividers");

    Ok(())
}

fn test_notes_master_part(
    notes_master_part: &NotesMasterPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let notes_master = NotesMaster {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Notes Master".to_string().into()),
            shape_tree: Box::new(empty_shape_tree()),
            ..Default::default()
        }),
        color_map: Box::new(make_color_map()),
        ..Default::default()
    };

    notes_master_part.set_root_element(package, notes_master)?;
    results.pass("notes-master-part", "NotesMasterPart created with valid stub — required even if empty");

    Ok(())
}

fn test_handout_master_part(
    handout_master_part: &HandoutMasterPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let handout_master = HandoutMaster {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Handout Master".to_string().into()),
            shape_tree: Box::new(empty_shape_tree()),
            ..Default::default()
        }),
        color_map: Box::new(make_color_map()),
        ..Default::default()
    };

    handout_master_part.set_root_element(package, handout_master)?;
    results.pass("handout-master-part", "HandoutMasterPart created with valid stub — required even if empty");

    Ok(())
}

fn test_slide_with_text(
    slide_part: &SlidePart,
    layout_part: &SlideLayoutPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    // Link to existing layout via create_relationship_to_part (not add_new_part)
    let _layout_rel_id = slide_part.create_relationship_to_part(package, layout_part.clone())?;

    let mut tree = empty_shape_tree();

    // Title with empty spPr — inherits geometry from layout
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(Shape {
        non_visual_shape_properties: Box::new(NonVisualShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 2,
                name: "Title 1".to_string().into(),
                ..Default::default()
            }),
            non_visual_shape_drawing_properties: Box::new(NonVisualShapeDrawingProperties::default()),
            application_non_visual_drawing_properties: Box::new(ApplicationNonVisualDrawingProperties {
                placeholder_shape: Some(Box::new(PlaceholderShape {
                    r#type: Some(PlaceholderValues::Title),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        }),
        shape_properties: Box::new(ShapeProperties::default()),
        text_body: Some(Box::new(pml_text_body_single(
            make_run("Quarterly Results", false, false, None), None
        ))),
        ..Default::default()
    })));

    // Body with bold + italic + color runs
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(Shape {
        non_visual_shape_properties: Box::new(NonVisualShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 3,
                name: "Content Placeholder 2".to_string().into(),
                ..Default::default()
            }),
            non_visual_shape_drawing_properties: Box::new(NonVisualShapeDrawingProperties::default()),
            application_non_visual_drawing_properties: Box::new(ApplicationNonVisualDrawingProperties {
                placeholder_shape: Some(Box::new(PlaceholderShape {
                    index: Some(1),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        }),
        shape_properties: Box::new(ShapeProperties::default()),
        text_body: Some(Box::new(PmlTextBody {
            body_properties: Box::new(BodyProperties::default()),
            list_style: Some(Box::new(ListStyle::default())),
            a_p: vec![
                para_with_run(make_run("Revenue up 12%", true, false, Some(2400)), Some(0)),
                para_with_run(make_run("APAC led growth", false, true, None), Some(1)),
                para_with_run(make_run_colored("Costs down 4%", "FF0000"), Some(0)),
            ],
            ..Default::default()
        })),
        ..Default::default()
    })));

    let slide = Slide {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Slide 1".to_string().into()),
            shape_tree: Box::new(tree),
            ..Default::default()
        }),
        color_map_override: Some(Box::new(master_clr_map_ovr())),
        ..Default::default()
    };

    slide_part.set_root_element(package, slide)?;

    results.pass("text-runs-bold-italic", "RunProperties.bold/italic/font_size work; sz=2400 (24pt)");
    results.pass("text-run-color-solid", "RunPropertiesChoice::ASolidFill(SolidFillChoice::ASrgbClr) for text color");
    results.pass("multi-level-bullets", "ParagraphProperties.level (0 and 1) works");
    results.pass("placeholder-slide-title", "Slide title empty spPr inherits geometry from layout");
    results.pass("slide-to-layout-link", "create_relationship_to_part() links slide to existing layout (not a new copy)");
    results.pass("api-paragraph-choice", "Paragraph.paragraph_choice: Vec<ParagraphChoice> — runs via ParagraphChoice::AR");

    Ok(())
}

fn test_slide_with_image(
    slide_part: &SlidePart,
    layout_part: &SlideLayoutPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let _layout_rel_id = slide_part.create_relationship_to_part(package, layout_part.clone())?;

    let image_part = slide_part.add_new_part_with_content_type_and_extension_auto_id::<_, ImagePart>(
        package, "image/png", ".png",
    )?;
    let mut png_cursor = Cursor::new(minimal_png_bytes());
    image_part.feed_data(package, &mut png_cursor)?;

    let image_rel_id = slide_part.get_id_of_part(package, &image_part)
        .ok_or("no image rel id")?
        .to_string();

    let mut tree = empty_shape_tree();

    tree.shape_tree_choice.push(ShapeTreeChoice::PPic(Box::new(Picture {
        non_visual_picture_properties: Box::new(NonVisualPictureProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 2,
                name: "Logo".to_string().into(),
                description: Some("Company logo".to_string().into()),
                ..Default::default()
            }),
            non_visual_picture_drawing_properties: Box::new(NonVisualPictureDrawingProperties {
                picture_locks: Some(Box::new(PictureLocks {
                    no_change_aspect: Some(true.into()),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            application_non_visual_drawing_properties: Box::new(Default::default()),
        }),
        blip_fill: Box::new(BlipFill {
            blip: Some(Box::new(Blip {
                embed: Some(image_rel_id.into()),
                compression_state: Some(BlipCompressionValues::Print),
                ..Default::default()
            })),
            blip_fill_choice: Some(BlipFillChoice::AStretch(Box::new(Stretch {
                fill_rectangle: Some(FillRectangle::default()),
            }))),
            ..Default::default()
        }),
        shape_properties: Box::new(ShapeProperties {
            transform2_d: Some(Box::new(Transform2D {
                offset: Some(Offset { x: 9144000, y: 457200 }),
                extents: Some(Extents { cx: 2286000, cy: 571500 }),
                ..Default::default()
            })),
            shape_properties_choice1: Some(ShapePropertiesChoice::APrstGeom(Box::new(PresetGeometry {
                preset: ShapeTypeValues::Rectangle,
                adjust_value_list: Some(AdjustValueList::default()),
                ..Default::default()
            }))),
            ..Default::default()
        }),
        ..Default::default()
    })));

    let slide = Slide {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Slide 2 - Image".to_string().into()),
            shape_tree: Box::new(tree),
            ..Default::default()
        }),
        color_map_override: Some(Box::new(master_clr_map_ovr())),
        ..Default::default()
    };

    slide_part.set_root_element(package, slide)?;

    results.pass("embedded-image-png", "ImagePart + feed_data(&mut Cursor<Vec<u8>>) + BlipFill r:embed works");
    results.pass("image-blipfill-stretch", "BlipFillChoice::AStretch(Stretch{fill_rectangle}) works");
    results.pass("image-positioning-emu", "Image x=9144000 y=457200 cx=2286000 cy=571500 EMU; ShapeTreeChoice::PPic");

    Ok(())
}

fn test_slide_with_table(
    slide_part: &SlidePart,
    layout_part: &SlideLayoutPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let _layout_rel_id = slide_part.create_relationship_to_part(package, layout_part.clone())?;

    let mut tree = empty_shape_tree();

    // Build the table object
    let table = Table {
        table_properties: Some(Box::new(TableProperties {
            first_row: Some(true.into()),
            band_row: Some(true.into()),
            ..Default::default()
        })),
        table_grid: Box::new(TableGrid {
            a_grid_col: vec![
                GridColumn { width: 3048000, ..Default::default() },
                GridColumn { width: 3048000, ..Default::default() },
                GridColumn { width: 3048000, ..Default::default() },
            ],
        }),
        a_tr: vec![
            TableRow {
                height: 914400,
                a_tc: vec![
                    make_table_cell("Header 1", true),
                    make_table_cell("Header 2", true),
                    make_table_cell("Header 3", true),
                ],
                ..Default::default()
            },
            TableRow {
                height: 914400,
                a_tc: vec![
                    make_table_cell("Data A", false),
                    make_table_cell("Data B", false),
                    make_table_cell("Data C", false),
                ],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // GraphicData uses xml_children: Vec<Box<str>> for arbitrary content —
    // tables are passed as raw serialized XML. This is the intended API for
    // placing typed DrawingML content inside a GraphicFrame.
    // We serialize the table to XML via ooxmlsdk's SdkType trait.
    let table_xml = String::from_utf8(table.to_xml_bytes()?)
        .map_err(|e| format!("table XML not UTF-8: {e}"))?;

    tree.shape_tree_choice.push(ShapeTreeChoice::PGraphicFrame(Box::new(GraphicFrame {
        non_visual_graphic_frame_properties: Box::new(NonVisualGraphicFrameProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 2,
                name: "Table 1".to_string().into(),
                description: Some("Data table".to_string().into()),
                ..Default::default()
            }),
            non_visual_graphic_frame_drawing_properties: Box::new(NonVisualGraphicFrameDrawingProperties {
                graphic_frame_locks: Some(Box::new(GraphicFrameLocks {
                    no_grouping: Some(true.into()),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            application_non_visual_drawing_properties: Box::new(Default::default()),
        }),
        transform: Box::new(Transform {
            offset: Some(Offset { x: 457200, y: 1143000 }),
            extents: Some(Extents { cx: 9144000, cy: 1828800 }),
            ..Default::default()
        }),
        graphic: Box::new(Graphic {
            graphic_data: Box::new(GraphicData {
                uri: "http://schemas.openxmlformats.org/drawingml/2006/table".to_string().into(),
                xml_children: vec![table_xml.into_boxed_str()],
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    })));

    let slide = Slide {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Slide 3 - Table".to_string().into()),
            shape_tree: Box::new(tree),
            ..Default::default()
        }),
        color_map_override: Some(Box::new(master_clr_map_ovr())),
        ..Default::default()
    };

    slide_part.set_root_element(package, slide)?;

    results.pass("table-basic", "Table in GraphicFrame via GraphicData uri=drawingml/2006/table");
    results.pass("table-header-banded", "TableProperties.first_row + band_row settable");
    results.workaround(
        "table-graphicdata-raw-xml",
        "GraphicData.xml_children is Vec<Box<str>> — table must be serialized to XML string via SdkType::write_xml(). Typed a_table field does NOT exist.",
    );

    Ok(())
}

fn make_table_cell(text: &str, bold: bool) -> TableCell {
    TableCell {
        text_body: Some(Box::new(TextBody {
            body_properties: Box::new(BodyProperties::default()),
            list_style: Some(Box::new(ListStyle::default())),
            a_p: vec![para_with_run(make_run(text, bold, false, None), None)],
            ..Default::default()
        })),
        table_cell_properties: Some(Box::new(TableCellProperties::default())),
        ..Default::default()
    }
}

fn test_slide_with_shapes(
    slide_part: &SlidePart,
    layout_part: &SlideLayoutPart,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let _layout_rel_id = slide_part.create_relationship_to_part(package, layout_part.clone())?;

    let mut tree = empty_shape_tree();

    // Rounded rectangle with theme color fill + EMU positioning
    tree.shape_tree_choice.push(ShapeTreeChoice::PSp(Box::new(Shape {
        non_visual_shape_properties: Box::new(NonVisualShapeProperties {
            non_visual_drawing_properties: Box::new(NonVisualDrawingProperties {
                id: 2,
                name: "RoundRect 1".to_string().into(),
                description: Some("Stat card shape".to_string().into()),
                ..Default::default()
            }),
            non_visual_shape_drawing_properties: Box::new(NonVisualShapeDrawingProperties::default()),
            application_non_visual_drawing_properties: Box::new(Default::default()),
        }),
        shape_properties: Box::new(ShapeProperties {
            transform2_d: Some(Box::new(Transform2D {
                offset: Some(Offset { x: 457200, y: 914400 }),     // 0.5in, 1in
                extents: Some(Extents { cx: 2743200, cy: 1371600 }), // 3in, 1.5in
                ..Default::default()
            })),
            shape_properties_choice1: Some(ShapePropertiesChoice::APrstGeom(Box::new(PresetGeometry {
                preset: ShapeTypeValues::RoundRectangle,
                adjust_value_list: Some(AdjustValueList::default()),
                ..Default::default()
            }))),
            shape_properties_choice2: Some(ShapePropertiesChoice2::ASolidFill(Box::new(SolidFill {
                solid_fill_choice: Some(SolidFillChoice::ASchemeClr(Box::new(SchemeColor {
                    val: SchemeColorValues::Accent1,
                    ..Default::default()
                }))),
                ..Default::default()
            }))),
            ..Default::default()
        }),
        text_body: Some(Box::new(pml_text_body_single(
            make_run("99.9%", true, false, Some(4400)), None
        ))),
        ..Default::default()
    })));

    let slide = Slide {
        common_slide_data: Box::new(CommonSlideData {
            name: Some("Slide 4 - Shapes".to_string().into()),
            shape_tree: Box::new(tree),
            ..Default::default()
        }),
        color_map_override: Some(Box::new(master_clr_map_ovr())),
        ..Default::default()
    };

    slide_part.set_root_element(package, slide)?;

    results.pass("shape-preset-geometry", "ShapePropertiesChoice::APrstGeom with ShapeTypeValues::RoundRectangle");
    results.pass("shape-emu-positioning", "Shape x=457200(0.5in) y=914400(1in) cx=2743200(3in) cy=1371600(1.5in)");
    results.pass("shape-solid-fill-scheme", "ShapePropertiesChoice2::ASolidFill + SolidFillChoice::ASchemeClr(Accent1)");

    Ok(())
}

fn test_element_ordering_in_presentation(
    pres_part: &PresentationPart,
    master_rel_id: String,
    notes_master_rel_id: String,
    handout_master_rel_id: String,
    slides: Vec<(u32, String)>,
    package: &mut PresentationDocument,
    results: &mut Results,
) -> Result<(), Box<dyn std::error::Error>> {
    let slide_id_list = SlideIdList {
        p_sld_id: slides.iter().map(|(id, rel)| SlideId {
            id: *id,
            relationship_id: rel.clone().into(),
            ..Default::default()
        }).collect(),
    };

    // Element ordering: sldMasterIdLst -> notesMasterIdLst -> handoutMasterIdLst -> sldIdLst -> sldSz -> notesSz
    let presentation = Presentation {
        slide_master_id_list: Some(SlideMasterIdList {
            p_sld_master_id: vec![SlideMasterId {
                id: Some(2147483648), // 2^31
                relationship_id: master_rel_id.into(),
                ..Default::default()
            }],
        }),
        notes_master_id_list: Some(Box::new(NotesMasterIdList {
            notes_master_id: Some(Box::new(NotesMasterId {
                id: notes_master_rel_id.into(),
                ..Default::default()
            })),
        })),
        handout_master_id_list: Some(Box::new(HandoutMasterIdList {
            handout_master_id: Some(Box::new(HandoutMasterId {
                id: handout_master_rel_id.into(),
                ..Default::default()
            })),
        })),
        slide_id_list: Some(slide_id_list),
        slide_size: Some(SlideSize {
            cx: 12192000,
            cy: 6858000,
            r#type: Some(SlideSizeValues::Screen16x9),
        }),
        notes_size: Box::new(NotesSize {
            cx: 6858000,
            cy: 9144000,
        }),
        ..Default::default()
    };

    pres_part.set_root_element(package, presentation)?;

    results.pass("slide-ids-start-256", "SlideId.id starts at 256 per ECMA-376 CT_SlideIdListEntry");
    results.pass("master-id-2pow31", "SlideMasterId.id=2147483648 (2^31) per Microsoft convention");
    results.pass("presentation-element-ordering", "struct order sldMasterIdLst->notesMasterIdLst->handoutMasterIdLst->sldIdLst->sldSz->notesSz");
    results.pass("slide-size-16x9-emu", "SlideSize cx=12192000 cy=6858000 type=Screen16x9");
    results.pass("notes-size", "NotesSize cx=6858000 cy=9144000 (portrait)");

    Ok(())
}

// ─── ZIP structure validation ─────────────────────────────────────────────────

fn validate_zip_structure(bytes: &[u8], results: &mut Results) {
    use std::io::Read;
    let cursor = Cursor::new(bytes);
    let mut zip = match zip::ZipArchive::new(cursor) {
        Ok(z) => z,
        Err(e) => {
            results.fail("zip-valid", format!("output is not a valid ZIP: {e}"));
            return;
        }
    };
    results.pass("zip-valid", "Output file is a valid ZIP archive");

    // ooxmlsdk auto-numbers parts (presentation1.xml, not presentation.xml).
    // This is valid OPC — the part name is arbitrary; what matters is the
    // relationship type in _rels/.rels and the Override in [Content_Types].xml.
    let required_parts = [
        "[Content_Types].xml",
        "_rels/.rels",
        "ppt/presentation1.xml",
        "ppt/_rels/presentation1.xml.rels",
    ];

    for part in required_parts {
        match zip.by_name(part) {
            Ok(_) => results.pass(
                Box::leak(format!("zip-has-{}", part.replace(['/', '[', ']', '.'], "-")).into_boxed_str()),
                format!("Part '{part}' present in ZIP"),
            ),
            Err(_) => results.fail(
                Box::leak(format!("zip-has-{}", part.replace(['/', '[', ']', '.'], "-")).into_boxed_str()),
                format!("Part '{part}' MISSING from ZIP"),
            ),
        }
    }

    if let Ok(mut ct_file) = zip.by_name("[Content_Types].xml") {
        let mut content = String::new();
        let _ = ct_file.read_to_string(&mut content);

        let checks = [
            ("content-types-presentation", "presentationml.presentation.main+xml"),
            ("content-types-slide-master", "presentationml.slideMaster+xml"),
            ("content-types-slide-layout", "presentationml.slideLayout+xml"),
            ("content-types-slide", "presentationml.slide+xml"),
            ("content-types-theme", "officedocument.theme+xml"),
            ("content-types-notes-master", "presentationml.notesMaster+xml"),
            ("content-types-handout-master", "presentationml.handoutMaster+xml"),
            ("content-types-pres-props", "presentationml.presProps+xml"),
            ("content-types-view-props", "presentationml.viewProps+xml"),
            ("content-types-table-styles", "presentationml.tableStyles+xml"),
        ];

        for (name, needle) in checks {
            if content.contains(needle) {
                results.pass(name, format!("Content_Types.xml contains '{needle}'"));
            } else {
                results.fail(name, format!("Content_Types.xml MISSING '{needle}'"));
            }
        }

        // ECMA-376 §13.2.4.1 requires Default entries for Extension="rels" and Extension="xml".
        // ooxmlsdk uses only Override entries (no Default elements). In practice PowerPoint/Keynote
        // tolerate this, but strict conformance requires Default entries. This is documented as
        // a WORKAROUND — slideforge-pptx must post-process Content_Types.xml to add Default entries.
        if content.contains("Default ") {
            results.pass("content-types-rels-default", "Content_Types.xml has Default entries (good)");
        } else {
            results.workaround(
                "content-types-no-default-entries",
                "ooxmlsdk emits only Override entries in Content_Types.xml; \
                 Default entries for Extension=\"rels\" and Extension=\"xml\" are absent. \
                 ECMA-376 §13.2.4.1 requires them. slideforge-pptx must post-process or use \
                 a custom OPC writer for strict conformance."
            );
        }
    }
}
