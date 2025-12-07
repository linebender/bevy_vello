use crate::{
    integrations::lottie::{
        LottieAssetVariant,
        player::events::{LottieOnAfterEvent, LottieOnCompletedEvent, LottieOnShowEvent},
    },
    prelude::*,
};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use tracing::debug;

pub fn on_add_lottie<A: LottieAssetVariant>(
    mut world: DeferredWorld<'_>,
    hook_context: HookContext,
) {
    {
        let world_guard = world.reborrow();
        bevy::camera::visibility::add_visibility_class::<A>(world_guard, hook_context);
    }

    let mut commands = world.commands();
    let mut entity_commands = commands.entity(hook_context.entity);

    entity_commands
        .observe(observe_pointer_enter::<A>)
        .observe(observe_pointer_exit::<A>)
        .observe(observe_pointer_click::<A>)
        .observe(observe_on_after::<A>)
        .observe(observe_on_complete::<A>)
        .observe(observe_on_show::<A>);
}

fn observe_pointer_enter<A: LottieAssetVariant>(
    trigger: On<Pointer<Over>>,
    mut lottie: Query<(&mut LottiePlayer<A>, &A)>,
    lotties: Res<Assets<VelloLottie>>,
) -> Result {
    debug!(entity = ?trigger.entity, "Lottie picking event: {}", trigger.event());
    let (mut player, lottie) = lottie.get_mut(trigger.entity)?;

    if player.stopped || player.states.len() <= 1 {
        return Ok(());
    }
    if lotties.get(lottie.asset_id()).is_none() {
        // Asset has not loaded yet and is therefore not visible. It would be odd to run transitions on assets that aren't visible.
        return Ok(());
    };

    if let Some(next_state) =
        player
            .state()
            .transitions
            .iter()
            .find_map(|transition| match transition {
                PlayerTransition::OnMouseEnter { state } => Some(*state),
                _ => None,
            })
    {
        player.next_state.replace(next_state);
    }

    Ok(())
}

fn observe_pointer_exit<A: LottieAssetVariant>(
    trigger: On<Pointer<Out>>,
    mut lottie: Query<(&mut LottiePlayer<A>, &A)>,
    lotties: Res<Assets<VelloLottie>>,
) -> Result {
    debug!(entity = ?trigger.entity, "Lottie picking event: {}", trigger.event());
    let (mut player, lottie) = lottie.get_mut(trigger.entity)?;

    if player.stopped || player.states.len() <= 1 {
        return Ok(());
    }
    if lotties.get(lottie.asset_id()).is_none() {
        // Asset has not loaded yet and is therefore not visible. It would be odd to run transitions on assets that aren't visible.
        return Ok(());
    };

    if let Some(next_state) =
        player
            .state()
            .transitions
            .iter()
            .find_map(|transition| match transition {
                PlayerTransition::OnMouseLeave { state } => Some(*state),
                _ => None,
            })
    {
        player.next_state.replace(next_state);
    }

    Ok(())
}

fn observe_pointer_click<A: LottieAssetVariant>(
    trigger: On<Pointer<Click>>,
    mut lottie: Query<(&mut LottiePlayer<A>, &A)>,
    lotties: Res<Assets<VelloLottie>>,
) -> Result {
    debug!(entity = ?trigger.entity, "Lottie picking event: {}", trigger.event());
    let (mut player, lottie) = lottie.get_mut(trigger.entity)?;

    if player.stopped || player.states.len() <= 1 {
        return Ok(());
    }
    if lotties.get(lottie.asset_id()).is_none() {
        // Asset has not loaded yet and is therefore not visible. It would be odd to run transitions on assets that aren't visible.
        return Ok(());
    };

    if let Some(next_state) =
        player
            .state()
            .transitions
            .iter()
            .find_map(|transition| match transition {
                PlayerTransition::OnMouseClick { state } => Some(*state),
                _ => None,
            })
    {
        player.next_state.replace(next_state);
    }

    Ok(())
}

fn observe_on_show<A: LottieAssetVariant>(
    trigger: On<LottieOnShowEvent>,
    mut lottie: Query<(&mut LottiePlayer<A>, &mut A)>,
    lotties: Res<Assets<VelloLottie>>,
) -> Result {
    let (mut player, lottie) = lottie.get_mut(trigger.entity)?;

    if player.stopped || player.states.len() <= 1 {
        return Ok(());
    }
    if lotties.get(lottie.asset_id()).is_none() {
        // Asset has not loaded yet and is therefore not visible. It would be odd to run transitions on assets that aren't visible.
        return Ok(());
    };

    player.next_state.replace(trigger.next_state);

    Ok(())
}

fn observe_on_after<A: LottieAssetVariant>(
    trigger: On<LottieOnAfterEvent>,
    mut lottie: Query<(&mut LottiePlayer<A>, &mut A)>,
    lotties: Res<Assets<VelloLottie>>,
) -> Result {
    let (mut player, lottie) = lottie.get_mut(trigger.entity)?;

    if player.stopped || player.states.len() <= 1 {
        return Ok(());
    }
    if lotties.get(lottie.asset_id()).is_none() {
        // Asset has not loaded yet and is therefore not visible. It would be odd to run transitions on assets that aren't visible.
        return Ok(());
    };

    player.next_state.replace(trigger.next_state);

    Ok(())
}

fn observe_on_complete<A: LottieAssetVariant>(
    trigger: On<LottieOnCompletedEvent>,
    mut lottie: Query<(&mut LottiePlayer<A>, &mut A)>,
    lotties: Res<Assets<VelloLottie>>,
) -> Result {
    let (mut player, lottie) = lottie.get_mut(trigger.entity)?;

    if player.stopped || player.states.len() <= 1 {
        return Ok(());
    }
    if lotties.get(lottie.asset_id()).is_none() {
        // Asset has not loaded yet and is therefore not visible. It would be odd to run transitions on assets that aren't visible.
        return Ok(());
    };

    player.next_state.replace(trigger.next_state);

    Ok(())
}
