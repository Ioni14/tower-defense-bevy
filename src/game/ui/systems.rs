use std::any::Any;
use bevy::prelude::*;

use crate::game::{GameState, UiState};
use crate::game::resources::{BuildTower, TowerType};
use crate::game::ui::components::BuildTowerAction;

pub fn spawn_action_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    build_action_bar(&mut commands, &asset_server);
}

pub fn build_action_bar(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(
            (
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::End,
                        padding: UiRect::all(Val::Px(16.0)),
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        // gap: Size::new(Val::Px(16.0), Val::Px(16.0)),
                        ..Style::DEFAULT
                    },
                    ..default()
                },
            )
        )
        .with_children(|parent: &mut ChildBuilder| {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(96.0), Val::Px(96.0)),
                        ..Style::DEFAULT
                    },
                    image: asset_server.load("ui/build_tower.png").into(),
                    ..default()
                },
                BuildTowerAction {
                    tower_type: TowerType::Arrow,
                },
            ));
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(96.0), Val::Px(96.0)),
                        ..Style::DEFAULT
                    },
                    image: asset_server.load("ui/build_tower_bomb.png").into(),
                    ..default()
                },
                BuildTowerAction {
                    tower_type: TowerType::Bomb,
                },
            ));
        });
}

pub fn interact_with_build_action(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &BuildTowerAction),
        (Changed<Interaction>),
    >,
    mut build_tower: ResMut<BuildTower>,
    mut game_next_state: ResMut<NextState<GameState>>,
    mut ui_next_state: ResMut<NextState<UiState>>,
    game_state: Res<State<GameState>>,
) {
    if let Ok((interaction, mut background_color, build_tower_action)) = button_query.get_single_mut() {
        match *interaction {
            Interaction::Clicked => {
                println!("clicked");
                if game_state.0 == GameState::Building {
                    *background_color = Color::rgb(0.7, 0.7, 0.7).into();
                    game_next_state.set(GameState::Playing);
                } else {
                    *background_color = Color::rgb(0.4, 0.8, 0.6).into();
                    game_next_state.set(GameState::Building);
                    build_tower.tower_type = build_tower_action.tower_type;
                    println!("building tower: {:?}", build_tower.tower_type);
                }
                ui_next_state.set(UiState::ChoosingAction);
            }
            Interaction::Hovered => {
                println!("hovered");
                *background_color = Color::rgb(0.7, 0.7, 0.7).into();
                ui_next_state.set(UiState::ChoosingAction);
            }
            Interaction::None => {
                println!("none");
                *background_color = Color::WHITE.into();
                ui_next_state.set(UiState::Nothing);
            }
        }
        if game_state.0 == GameState::Building {
            *background_color = Color::rgb(0.4, 0.8, 0.6).into();
        }
    }
}
