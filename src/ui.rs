use bevy::{
    color::palettes::css::{BLACK, WHITE},
    prelude::*,
    window::{CursorIcon, SystemCursorIcon},
};

use crate::{
    camera,
    selection::{Action, PerformingAction, Select, Selectable, Selected},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (spawn_ui.before(camera), setup_cursor))
        .add_observer(on_select);
}

#[derive(Resource)]
struct Font(TextFont);

fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = Font(TextFont {
        font: asset_server.load("pixelify_sans.ttf").into(),
        font_size: FontSize::Px(30.),
        ..default()
    });

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::RowReverse,
                ..default()
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .id();

    let side_panel = commands
        .spawn((
            ChildOf(root),
            SidePanel,
            BackgroundColor(BLACK.into()),
            Node {
                width: px(300),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            },
        ))
        .id();

    let _portrait = commands
        .spawn((
            ChildOf(side_panel),
            Node {
                width: px(270),
                height: px(270),
                ..default()
            },
            ImageNode { ..default() },
            Portrait,
        ))
        .id();

    let _selection_text = commands
        .spawn((
            ChildOf(side_panel),
            Text::new(""),
            font.0.clone(),
            SelectionText,
        ))
        .id();

    commands.insert_resource(font);
}

#[derive(Component)]
struct Portrait;

#[derive(Component)]
struct SelectionText;

#[derive(Component)]
struct SidePanel;

fn on_select(
    _: On<Select>,
    selectables: Query<&Selectable>,
    mut portrait: Single<&mut ImageNode, With<Portrait>>,
    mut selection_text: Single<&mut Text, With<SelectionText>>,
    selection_actions: Query<Entity, With<SelectionAction>>,
    mut commands: Commands,
    font: Res<Font>,
    side_panel: Single<Entity, With<SidePanel>>,
    selected: Res<Selected>,
) {
    let selectable = selectables.get(selected.0).else_error()?;
    portrait.image = selectable.portrait.clone();
    **selection_text = Text::new(format!("Selected: {}", selectable.name));

    for selection_action in selection_actions {
        commands.entity(selection_action).despawn();
    }

    for action in selectable.actions.iter().cloned() {
        let name = action.name();
        let button = commands
            .spawn((
                ChildOf(*side_panel),
                SelectionAction(action),
                Node {
                    width: percent(80),
                    height: px(40),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(WHITE.into()),
                Outline {
                    width: px(2),
                    offset: px(0),
                    color: WHITE.into(),
                },
            ))
            .observe(
                |on: On<PointerClick>,
                 selection_actions: Query<&SelectionAction>,
                 background_colours: Query<
                    (Entity, &mut BackgroundColor, &Children),
                    With<SelectionAction>,
                >,
                 mut text_colours: Query<&mut TextColor>,
                 mut performing_action: ResMut<PerformingAction>,
                 cursor_icons: Query<&mut CursorIcon>| {
                    let selection_action = selection_actions.get(on.entity).else_error()?;
                    performing_action.0 = if let Some(performing_action) = &performing_action.0
                        && performing_action == &selection_action.0
                    {
                        for mut cursor_icon in cursor_icons {
                            *cursor_icon = CursorIcon::System(SystemCursorIcon::Default);
                        }
                        None
                    } else {
                        for mut cursor_icon in cursor_icons {
                            *cursor_icon = CursorIcon::System(SystemCursorIcon::NotAllowed);
                        }
                        Some(selection_action.0.clone())
                    };

                    for (entity, mut background_colour, children) in background_colours {
                        if entity == on.entity {
                            continue;
                        }

                        let mut text_colour = text_colours
                            .get_mut(*children.first().else_error()?)
                            .else_error()?;

                        background_colour.0 = WHITE.into();
                        text_colour.0 = BLACK.into();
                    }
                },
            )
            .observe(
                |on: On<PointerOver>,
                 mut background_colours: Query<(&mut BackgroundColor, &Children)>,
                 mut text_colours: Query<&mut TextColor>| {
                    let (mut background_colour, children) =
                        background_colours.get_mut(on.entity).else_error()?;
                    let mut text_colour = text_colours
                        .get_mut(*children.first().else_error()?)
                        .else_error()?;

                    background_colour.0 = BLACK.into();
                    text_colour.0 = WHITE.into();
                },
            )
            .observe(
                |on: On<PointerOut>,
                 selection_actions: Query<&SelectionAction>,
                 mut background_colours: Query<(&mut BackgroundColor, &Children)>,
                 mut text_colours: Query<&mut TextColor>,
                 performing_action: Res<PerformingAction>| {
                    let selection_action = selection_actions.get(on.entity).else_error()?;
                    if let Some(performing_action) = &performing_action.0
                        && performing_action == &selection_action.0
                    {
                        return;
                    }

                    let (mut background_colour, children) =
                        background_colours.get_mut(on.entity).else_error()?;
                    let mut text_colour = text_colours
                        .get_mut(*children.first().else_error()?)
                        .else_error()?;

                    background_colour.0 = WHITE.into();
                    text_colour.0 = BLACK.into();
                },
            )
            .id();
        let _text = commands
            .spawn((
                ChildOf(button),
                Text::new(name),
                font.0.clone(),
                TextColor::BLACK,
            ))
            .id();
    }
}

#[derive(Component)]
pub struct SelectionAction(Action);

fn setup_cursor(window: Single<Entity, With<Window>>, mut commands: Commands) {
    commands
        .entity(*window)
        .insert(CursorIcon::System(SystemCursorIcon::Default));
}
