#![allow(clippy::uninlined_format_args)]
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::sync::Mutex;

static BEZIER_ID: Mutex<usize> = Mutex::new(0);

pub fn new_id() -> usize {
    let mut bezier_id = BEZIER_ID.lock().unwrap();
    *bezier_id += 1;
    *bezier_id
}

#[derive(Clone)]
pub struct BezierShape {
    pub shape_type: BezierShapeType,
    pub id: usize,
    pub point: Option<Vec2>,
}

#[derive(Clone, Default)]
pub enum BezierShapeType {
    #[default]
    Start,
    ControlStart,
    ControlEnd,
    End,
    Line,
    BezierLine,
}

impl std::fmt::Display for BezierShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BezierShapeType::Start => write!(f, "Start"),
            BezierShapeType::ControlStart => write!(f, "ControlStart"),
            BezierShapeType::ControlEnd => write!(f, "ControlEnd"),
            BezierShapeType::End => write!(f, "End"),
            BezierShapeType::Line => write!(f, "Line"),
            BezierShapeType::BezierLine => write!(f, "BezierLine"),
        }
    }
}

#[derive(Resource, Default)]
pub struct BezierDrag {
    pub bezier_id: usize,
    pub entity: Option<Entity>,
    pub dragging: BezierShapeType,
    pub start_click: Option<Vec2>,
    pub a: Option<Vec2>,
    pub b: Option<Vec2>,
    pub c: Option<Vec2>,
    pub d: Option<Vec2>,
}

impl BezierDrag {
    pub fn clear_drag(&mut self) {
        self.bezier_id = 0;
        self.entity = None;
        self.start_click = None;
        self.a = None;
        self.b = None;
        self.c = None;
        self.d = None;
    }
    pub fn add_delta(&mut self, delta: Vec2) {
        match self.dragging {
            BezierShapeType::Start => {
                let point = self.a.unwrap();
                self.a = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
            }
            BezierShapeType::ControlStart => {
                let point = self.b.unwrap();
                self.b = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
            }
            BezierShapeType::ControlEnd => {
                let point = self.c.unwrap();
                self.c = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
            }
            BezierShapeType::End => {
                let point = self.d.unwrap();
                self.d = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
            }
            BezierShapeType::Line => {}
            BezierShapeType::BezierLine => {
                let point = self.a.unwrap();
                self.a = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
                let point = self.b.unwrap();
                self.b = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
                let point = self.c.unwrap();
                self.c = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
                let point = self.d.unwrap();
                self.d = Some(Vec2::new(point.x + delta.x, point.y - delta.y));
            }
        }
    }
}

#[derive(Clone, Component)]
pub enum ShapeType {
    Intersection,
    Main,
    Sketch,
    Bezier(BezierShape),
}

impl std::fmt::Display for ShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeType::Intersection => write!(f, "Intersection"),
            ShapeType::Main => write!(f, "Main"),
            ShapeType::Sketch => write!(f, "Sketch"),
            ShapeType::Bezier(_bezier_shape) => write!(f, "Bezier"),
        }
    }
}

#[derive(Resource)]
pub struct BezierStyle {
    pub intersection_color: Color,
    pub intersection_radius: f32,
    pub sketch_color: Color,
    pub bezier_stroke_width: f32,
    pub sketch_stroke_width: f32,
    pub bezier_line_color: Color,
}

impl Default for BezierStyle {
    fn default() -> Self {
        Self {
            intersection_color: Color::srgba(1.0, 0.0, 0.0, 1.0),
            sketch_color: Color::srgba(0.5, 0.5, 0.5, 1.0),
            intersection_radius: 6.0,
            bezier_stroke_width: 4.0,
            sketch_stroke_width: 1.0,
            bezier_line_color: Color::srgba_u8(200, 172, 110, 255),
        }
    }
}

pub struct BezierPlugin;

impl Plugin for BezierPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BezierStyle::default())
            .insert_resource(BezierDrag::default());
    }
}

pub fn bezier_open(
    style: &BezierStyle,
    id: usize,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
) -> Vec<(Shape, ShapeType)> {
    let radius = style.intersection_radius - 1.0;
    let stroke = style.sketch_stroke_width;
    let thick_stroke_width = style.bezier_stroke_width;
    let i_color = style.intersection_color;
    let mut shapes = Vec::new();
    let color = style.sketch_color;
    let bezier_color = style.bezier_line_color;

    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Circle { radius, center: a })
            .fill(i_color)
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::Start,
            id,
            point: Some(a),
        }),
    ));

    let path = ShapePath::new().move_to(a).cubic_bezier_to(b, c, d);
    shapes.push((
        ShapeBuilder::with(&path)
            .stroke((bezier_color, thick_stroke_width))
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::BezierLine,
            id,
            point: None,
        }),
    ));

    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Line(a, b))
            .stroke((color, stroke))
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::Line,
            id,
            point: None,
        }),
    ));
    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Circle { radius, center: b })
            .fill(i_color)
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::ControlStart,
            id,
            point: Some(b),
        }),
    ));
    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Line(b, c))
            .stroke((color, stroke))
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::Line,
            id,
            point: None,
        }),
    ));
    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Circle { radius, center: c })
            .fill(i_color)
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::ControlEnd,
            id,
            point: Some(c),
        }),
    ));

    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Line(c, d))
            .stroke((color, stroke))
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::Line,
            id,
            point: None,
        }),
    ));

    shapes.push((
        ShapeBuilder::new()
            .add(&shapes::Circle { radius, center: d })
            .fill(i_color)
            .build(),
        ShapeType::Bezier(BezierShape {
            shape_type: BezierShapeType::End,
            id,
            point: Some(d),
        }),
    ));

    shapes
}

pub fn drag_start(
    click: Trigger<Pointer<DragStart>>,
    query: Query<(Entity, &mut Shape, &ShapeType)>,
    mut commands: Commands,
    mut drag: ResMut<BezierDrag>,
) {
    let Ok((drag_entity, _shape, drag_shape_type)) = query.get(click.target) else {
        return;
    };

    commands.entity(drag_entity).insert(Visibility::Hidden);
    let (bezier_id, part_drag) = if let ShapeType::Bezier(bezier_shape) = drag_shape_type {
        (bezier_shape.id, bezier_shape.shape_type.clone())
    } else {
        return;
    };

    drag.bezier_id = bezier_id;
    drag.entity = Some(drag_entity);
    drag.start_click = Some(click.event().pointer_location.position);
    drag.dragging = part_drag;

    // find the bezier points with id
    for (_entity, _shape, shape_type) in query.iter() {
        if let ShapeType::Bezier(bezier_shape) = shape_type {
            if bezier_id == bezier_shape.id {
                match bezier_shape.shape_type {
                    BezierShapeType::Start => {
                        let point = bezier_shape.point.unwrap();
                        drag.a = Some(point);
                    }
                    BezierShapeType::ControlStart => {
                        let point = bezier_shape.point.unwrap();
                        drag.b = Some(point);
                    }
                    BezierShapeType::ControlEnd => {
                        let point = bezier_shape.point.unwrap();
                        drag.c = Some(point);
                    }
                    BezierShapeType::End => {
                        let point = bezier_shape.point.unwrap();
                        drag.d = Some(point);
                    }
                    BezierShapeType::Line => {}
                    BezierShapeType::BezierLine => {}
                }
            }
        }
    }
    assert!(drag.a.is_some());
    assert!(drag.b.is_some());
    assert!(drag.c.is_some());
    assert!(drag.d.is_some());
}

pub fn bezier_drag(
    click: Trigger<Pointer<Drag>>,
    mut commands: Commands,
    query: Query<(Entity, &mut Shape, &ShapeType)>,
    mut drag: ResMut<BezierDrag>,
    style: Res<BezierStyle>,
) {
    let Ok((drag_entity, _shape, _drag_shape_type)) = query.get(click.target) else {
        return;
    };
    for (entity, _, shape_type) in query.iter() {
        if let ShapeType::Bezier(bezier) = shape_type {
            if bezier.id == drag.bezier_id && drag_entity != entity {
                commands.entity(entity).despawn();
            }
        }
    }
    drag.add_delta(click.delta);
    let shapes = bezier_open(
        &style,
        drag.bezier_id,
        drag.a.unwrap(),
        drag.b.unwrap(),
        drag.c.unwrap(),
        drag.d.unwrap(),
    );
    for (n, (shape, shape_type)) in shapes.into_iter().enumerate() {
        commands
            .spawn((
                shape,
                shape_type,
                Pickable::default(),
                Transform::from_xyz(0.0, 0.0, n as f32 * 0.01),
            ))
            .observe(drag_start)
            .observe(bezier_drag)
            .observe(drag_end);
    }
}

pub fn drag_end(
    _click: Trigger<Pointer<DragEnd>>,
    _query: Query<(Entity, &mut Shape, &ShapeType, &Transform)>,
    mut commands: Commands,
    mut drag: ResMut<BezierDrag>,
) {
    commands.entity(drag.entity.unwrap()).despawn();
    drag.clear_drag();
}

