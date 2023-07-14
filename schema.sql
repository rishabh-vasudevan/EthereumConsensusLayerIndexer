create table if not exists Attestations (
    id integer PRIMARY KEY GENERATED ALWAYS as Identity,
    epoch_id integer not null,
    slot_id integer not null,
    committee_id integer not null,
    validator_id text not null,
    attested boolean not null 
);