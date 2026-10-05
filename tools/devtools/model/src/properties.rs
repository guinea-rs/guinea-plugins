//! Native properties sorted the way the browser's devtools sort styles: into
//! clusters by what they mean, and read as what they are - a colour, a box,
//! a named enumeration value - rather than as the text the backend sends.

use std::collections::HashMap;

use guinea_devtools_protocol::native::Property;

/// What a property is about. The sidebar has one tab per cluster.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cluster {
    Layout,
    Appearance,
    Text,
    Transform,
    Interaction,
    Content,
    Accessibility,
    Other,
}

impl Cluster {
    pub const ALL: [Cluster; 8] = [
        Cluster::Layout,
        Cluster::Appearance,
        Cluster::Text,
        Cluster::Transform,
        Cluster::Interaction,
        Cluster::Content,
        Cluster::Accessibility,
        Cluster::Other,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Cluster::Layout => "Layout",
            Cluster::Appearance => "Appearance",
            Cluster::Text => "Text",
            Cluster::Transform => "Transform",
            Cluster::Interaction => "Interaction",
            Cluster::Content => "Content",
            Cluster::Accessibility => "Accessibility",
            Cluster::Other => "Other",
        }
    }

    pub fn of(property: &Property) -> Cluster {
        let name = property.name.as_str();
        let owner = name.split_once('.').map(|(owner, _)| owner);

        match owner {
            Some("Grid" | "Canvas" | "RelativePanel" | "VariableSizedWrapGrid") => {
                return Cluster::Layout;
            }
            Some("Typography") => return Cluster::Text,
            Some("AutomationProperties") => return Cluster::Accessibility,
            Some("ToolTipService" | "FlyoutBase") => return Cluster::Content,
            Some("ScrollViewer" | "VirtualizingStackPanel") => return Cluster::Interaction,
            Some(_) => return Cluster::Other,
            None => {}
        }

        const LAYOUT: &[&str] = &[
            "Width",
            "Height",
            "MinWidth",
            "MinHeight",
            "MaxWidth",
            "MaxHeight",
            "Margin",
            "Padding",
            "HorizontalAlignment",
            "VerticalAlignment",
            "HorizontalContentAlignment",
            "VerticalContentAlignment",
            "ActualWidth",
            "ActualHeight",
            "ActualSize",
            "ActualOffset",
            "DesiredSize",
            "RenderSize",
            "UseLayoutRounding",
            "FlowDirection",
            "Orientation",
            "Spacing",
            "RowDefinitions",
            "ColumnDefinitions",
            "RowSpacing",
            "ColumnSpacing",
        ];
        const APPEARANCE: &[&str] = &[
            "Background",
            "Foreground",
            "BorderBrush",
            "BorderThickness",
            "CornerRadius",
            "Opacity",
            "Visibility",
            "Fill",
            "Stroke",
            "StrokeThickness",
            "Shadow",
            "Clip",
            "BackgroundSizing",
            "BackgroundTransition",
            "OpacityTransition",
            "ActualTheme",
            "RequestedTheme",
            "CompositeMode",
            "Lights",
            "HighContrastAdjustment",
        ];
        const TEXT: &[&str] = &[
            "Text",
            "FontFamily",
            "FontSize",
            "FontWeight",
            "FontStyle",
            "FontStretch",
            "CharacterSpacing",
            "LineHeight",
            "LineStackingStrategy",
            "TextWrapping",
            "TextTrimming",
            "TextAlignment",
            "TextLineBounds",
            "MaxLines",
            "IsTextScaleFactorEnabled",
            "OpticalMarginAlignment",
            "TextDecorations",
            "TextReadingOrder",
            "IsTextSelectionEnabled",
        ];
        const TRANSFORM: &[&str] = &[
            "RenderTransform",
            "RenderTransformOrigin",
            "Projection",
            "Transform3D",
            "TransformMatrix",
            "Rotation",
            "RotationAxis",
            "RotationTransition",
            "Scale",
            "ScaleTransition",
            "Translation",
            "TranslationTransition",
            "CenterPoint",
            "RasterizationScale",
        ];
        const CONTENT: &[&str] = &[
            "Content",
            "ContentTemplate",
            "ContentTemplateSelector",
            "ContentTransitions",
            "Child",
            "Children",
            "Style",
            "Template",
            "DataContext",
            "Tag",
            "Name",
            "Parent",
            "Resources",
            "ContextFlyout",
            "Transitions",
            "ChildrenTransitions",
            "Triggers",
            "Source",
        ];

        if LAYOUT.contains(&name) {
            Cluster::Layout
        } else if APPEARANCE.contains(&name) {
            Cluster::Appearance
        } else if TEXT.contains(&name) {
            Cluster::Text
        } else if TRANSFORM.contains(&name) {
            Cluster::Transform
        } else if CONTENT.contains(&name) {
            Cluster::Content
        } else if name.starts_with("Is")
            || name.starts_with("Can")
            || name.starts_with("Allow")
            || name.starts_with("XYFocus")
            || name.starts_with("KeyTip")
            || name.starts_with("AccessKey")
            || name.starts_with("KeyboardAccelerator")
            || name.starts_with("Focus")
            || name.starts_with("Tab")
            || name.starts_with("Manipulation")
            || name.starts_with("Pointer")
            || name == "ExitDisplayModeOnAccessKeyInvoked"
            || name == "UseSystemFocusVisuals"
            || name == "ProtectedCursor"
        {
            Cluster::Interaction
        } else {
            Cluster::Other
        }
    }
}

/// A named part of a cluster; its properties in the order they matter.
#[derive(Clone, Copy, Debug)]
pub struct Section {
    pub title: &'static str,
    pub names: &'static [&'static str],
}

impl Cluster {
    /// Its sections, the ones looked at most first. What none of them names
    /// is the rest, shown last and folded.
    pub fn sections(self) -> &'static [Section] {
        match self {
            Cluster::Layout => &[
                Section {
                    title: "Size",
                    names: &[
                        "Width",
                        "Height",
                        "ActualWidth",
                        "ActualHeight",
                        "MinWidth",
                        "MinHeight",
                        "MaxWidth",
                        "MaxHeight",
                    ],
                },
                Section {
                    title: "Spacing",
                    names: &[
                        "Margin",
                        "Padding",
                        "Spacing",
                        "RowSpacing",
                        "ColumnSpacing",
                    ],
                },
                Section {
                    title: "Alignment",
                    names: &[
                        "HorizontalAlignment",
                        "VerticalAlignment",
                        "HorizontalContentAlignment",
                        "VerticalContentAlignment",
                        "Orientation",
                        "FlowDirection",
                    ],
                },
                Section {
                    title: "Position",
                    names: &[
                        "Grid.Row",
                        "Grid.Column",
                        "Grid.RowSpan",
                        "Grid.ColumnSpan",
                        "RowDefinitions",
                        "ColumnDefinitions",
                        "Canvas.Left",
                        "Canvas.Top",
                        "Canvas.ZIndex",
                    ],
                },
            ],
            Cluster::Appearance => &[
                Section {
                    title: "Visibility",
                    names: &["Visibility", "Opacity"],
                },
                Section {
                    title: "Brushes",
                    names: &["Background", "Foreground", "BorderBrush", "Fill", "Stroke"],
                },
                Section {
                    title: "Border",
                    names: &[
                        "BorderThickness",
                        "CornerRadius",
                        "StrokeThickness",
                        "BackgroundSizing",
                    ],
                },
                Section {
                    title: "Theme",
                    names: &["RequestedTheme", "ActualTheme"],
                },
            ],
            Cluster::Text => &[
                Section {
                    title: "Text",
                    names: &[
                        "Text",
                        "TextWrapping",
                        "TextTrimming",
                        "TextAlignment",
                        "MaxLines",
                    ],
                },
                Section {
                    title: "Font",
                    names: &[
                        "FontFamily",
                        "FontSize",
                        "FontWeight",
                        "FontStyle",
                        "FontStretch",
                    ],
                },
                Section {
                    title: "Lines",
                    names: &[
                        "LineHeight",
                        "CharacterSpacing",
                        "LineStackingStrategy",
                        "TextLineBounds",
                    ],
                },
            ],
            Cluster::Transform => &[
                Section {
                    title: "Render",
                    names: &["RenderTransform", "RenderTransformOrigin"],
                },
                Section {
                    title: "Composition",
                    names: &[
                        "Translation",
                        "Rotation",
                        "RotationAxis",
                        "Scale",
                        "CenterPoint",
                        "TransformMatrix",
                    ],
                },
            ],
            Cluster::Interaction => &[
                Section {
                    title: "State",
                    names: &[
                        "IsEnabled",
                        "IsHitTestVisible",
                        "IsTabStop",
                        "TabIndex",
                        "IsFocusEngaged",
                    ],
                },
                Section {
                    title: "Focus",
                    names: &[
                        "FocusState",
                        "AllowFocusOnInteraction",
                        "AllowFocusWhenDisabled",
                        "UseSystemFocusVisuals",
                        "TabFocusNavigation",
                    ],
                },
            ],
            Cluster::Content => &[
                Section {
                    title: "Content",
                    names: &["Content", "Child", "Children", "Source"],
                },
                Section {
                    title: "Templates",
                    names: &[
                        "Style",
                        "Template",
                        "ContentTemplate",
                        "ContentTemplateSelector",
                    ],
                },
                Section {
                    title: "Data",
                    names: &["DataContext", "Name", "Tag"],
                },
            ],
            Cluster::Accessibility => &[Section {
                title: "Name",
                names: &[
                    "AutomationProperties.Name",
                    "AutomationProperties.AutomationId",
                    "AutomationProperties.HelpText",
                    "AutomationProperties.LabeledBy",
                ],
            }],
            Cluster::Other => &[],
        }
    }

    /// Which of [`Cluster::sections`] names `property`, and where in it.
    pub fn place(self, property: &Property) -> Option<(usize, usize)> {
        self.sections()
            .iter()
            .enumerate()
            .find_map(|(at, section)| {
                let within = section
                    .names
                    .iter()
                    .position(|name| *name == property.name)?;
                Some((at, within))
            })
    }
}

/// The winning value of each property, those another source overrides left
/// out.
pub fn winning(properties: &[Property]) -> impl Iterator<Item = &Property> {
    properties.iter().filter(|property| !property.overridden)
}

pub fn find<'a>(properties: &'a [Property], name: &str) -> Option<&'a Property> {
    winning(properties).find(|property| property.name == name)
}

/// `left, top, right, bottom`, from XAML's one-, two- or four-number forms.
pub fn thickness(value: &str) -> Option<[f64; 4]> {
    let numbers: Vec<f64> = value
        .split(',')
        .map(|part| part.trim().parse())
        .collect::<Result<_, _>>()
        .ok()?;

    match numbers.as_slice() {
        [all] => Some([*all; 4]),
        [horizontal, vertical] => Some([*horizontal, *vertical, *horizontal, *vertical]),
        [left, top, right, bottom] => Some([*left, *top, *right, *bottom]),
        _ => None,
    }
}

/// `[r, g, b, a]` from a colour as XAML writes one: `#AARRGGBB`, `#RRGGBB`, or
/// a name like `White`.
pub fn color(value: &str) -> Option<[u8; 4]> {
    let value = value.trim();

    if let Some(hex) = value.strip_prefix('#') {
        let byte = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok();
        return match hex.len() {
            8 => Some([byte(2)?, byte(4)?, byte(6)?, byte(0)?]),
            6 => Some([byte(0)?, byte(2)?, byte(4)?, 255]),
            _ => None,
        };
    }

    csscolorparser::parse(value)
        .ok()
        .map(|color| color.to_rgba8())
}

/// What an enumeration value is called: `Stretch` for `3`.
pub fn enum_name<'a>(
    enums: &'a HashMap<String, Vec<(i32, String)>>,
    property: &Property,
) -> Option<&'a str> {
    let number: i32 = property.value.parse().ok()?;
    enums
        .get(&property.value_type)?
        .iter()
        .find(|(value, _)| *value == number)
        .map(|(_, name)| name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> Property {
        Property {
            name: name.into(),
            ..Property::default()
        }
    }

    #[test]
    fn properties_land_in_their_clusters() {
        assert_eq!(Cluster::of(&named("Margin")), Cluster::Layout);
        assert_eq!(Cluster::of(&named("Grid.Row")), Cluster::Layout);
        assert_eq!(Cluster::of(&named("Background")), Cluster::Appearance);
        assert_eq!(Cluster::of(&named("Typography.Kerning")), Cluster::Text);
        assert_eq!(Cluster::of(&named("Rotation")), Cluster::Transform);
        assert_eq!(Cluster::of(&named("IsTabStop")), Cluster::Interaction);
        assert_eq!(
            Cluster::of(&named("AutomationProperties.Name")),
            Cluster::Accessibility
        );
        assert_eq!(Cluster::of(&named("Child")), Cluster::Content);
        assert_eq!(Cluster::of(&named("Language")), Cluster::Other);
    }

    #[test]
    fn what_matters_comes_first_and_the_rest_is_left_over() {
        assert_eq!(
            Cluster::Appearance.place(&named("Visibility")),
            Some((0, 0))
        );
        assert_eq!(
            Cluster::Appearance.place(&named("Background")),
            Some((1, 0))
        );
        assert_eq!(
            Cluster::Appearance.place(&named("HighContrastAdjustment")),
            None
        );
        assert_eq!(Cluster::Other.place(&named("Language")), None);
    }

    #[test]
    fn thicknesses_read_in_every_form() {
        assert_eq!(thickness("4"), Some([4.0; 4]));
        assert_eq!(thickness("1,2"), Some([1.0, 2.0, 1.0, 2.0]));
        assert_eq!(thickness("0,8,12.5,0"), Some([0.0, 8.0, 12.5, 0.0]));
        assert_eq!(thickness(""), None);
    }

    #[test]
    fn colours_read_as_xaml_writes_them() {
        assert_eq!(color("#80FF0000"), Some([255, 0, 0, 128]));
        assert_eq!(color("#00FF00"), Some([0, 255, 0, 255]));
        assert_eq!(color("White"), Some([255, 255, 255, 255]));
        assert_eq!(color("nonsense"), None);
    }

    #[test]
    fn enumeration_values_have_names() {
        let enums = HashMap::from([(
            "Microsoft.UI.Xaml.HorizontalAlignment".to_string(),
            vec![(0, "Left".to_string()), (3, "Stretch".to_string())],
        )]);
        let property = Property {
            value: "3".into(),
            value_type: "Microsoft.UI.Xaml.HorizontalAlignment".into(),
            ..Property::default()
        };

        assert_eq!(enum_name(&enums, &property), Some("Stretch"));
    }
}
