summarized in [cheatbook](https://bevy-cheatbook.github.io/programming/schedules.html#the-main-schedule) and
[docs](https://docs.rs/bevy/latest/bevy/app/struct.Main.html)

most relevant schedules:

    First

    PreUpdate

    StateTransition

    Update

    PostUpdate (parent-child transform propagation)

    Last

notes:

    First

    PreUpdate
        anything that needs to read info as user saw
        anything that performs state transition
        any input processing

    StateTransition

    Update
        reaction to user input/events
        update position according to changed schematic
        prune set (prune, insert netid, connected graphs)

    PostUpdate (parent-child transform propagation)

    Last

