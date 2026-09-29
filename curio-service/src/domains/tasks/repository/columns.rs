//! Column enums for task tables and the names task queries derive.
use crate::adapters::postgres::query::sql_columns;

sql_columns! {
    pub(super) enum TaskColumn {
        Id => "id",
        OwnerId => "owner_id",
        SpaceId => "space_id",
        ParentId => "parent_id",
        Title => "title",
        Description => "description",
        Status => "status",
        Priority => "priority",
        Flagged => "flagged",
        DueOn => "due_on",
        DueAt => "due_at",
        DueTimezone => "due_timezone",
        CompletedAt => "completed_at",
        SortOrder => "sort_order",
        Version => "version",
        CreatedAt => "created_at",
        UpdatedAt => "updated_at",
    }
}

sql_columns! {
    pub(super) enum TagColumn {
        Id => "id",
        OwnerId => "owner_id",
        Name => "name",
        NameKey => "name_key",
        CreatedAt => "created_at",
    }
}

sql_columns! {
    // These names mirror the database foreign-key columns.
    #[allow(clippy::enum_variant_names)]
    pub(super) enum TagAssignmentColumn {
        OwnerId => "owner_id",
        TaskId => "task_id",
        TagId => "tag_id",
    }
}

sql_columns! {
    pub(super) enum CommentColumn {
        Id => "id",
        OwnerId => "owner_id",
        TaskId => "task_id",
        AuthorId => "author_id",
        Body => "body",
        CreatedAt => "created_at",
        EditedAt => "edited_at",
    }
}

sql_columns! {
    pub(super) enum LinkColumn {
        Id => "id",
        OwnerId => "owner_id",
        TaskId => "task_id",
        Kind => "kind",
        Label => "label",
        Url => "url",
        NoteId => "note_id",
        SortOrder => "sort_order",
        CreatedAt => "created_at",
        UpdatedAt => "updated_at",
    }
}

sql_columns! {
    /// PostgreSQL's catalog of IANA zones.
    pub(super) enum TimezoneColumn {
        Name => "name",
    }
}

sql_columns! {
    /// Names produced inside task queries: CTE, row-source and projection aliases.
    pub(super) enum DerivedColumn {
        Id => "id",
        Rank => "rank",
        Depth => "depth",
        CommentCount => "comment_count",
        ChildCount => "child_count",
    }
}

/// Stored task fields read into `TaskRow`, before the derived counts.
pub(super) const TASK_COLUMNS: [TaskColumn; 16] = [
    TaskColumn::Id,
    TaskColumn::SpaceId,
    TaskColumn::ParentId,
    TaskColumn::Title,
    TaskColumn::Description,
    TaskColumn::Status,
    TaskColumn::Priority,
    TaskColumn::Flagged,
    TaskColumn::DueOn,
    TaskColumn::DueAt,
    TaskColumn::DueTimezone,
    TaskColumn::CompletedAt,
    TaskColumn::SortOrder,
    TaskColumn::Version,
    TaskColumn::CreatedAt,
    TaskColumn::UpdatedAt,
];

/// Fields read into `TagRow`.
pub(super) const TAG_COLUMNS: [TagColumn; 3] =
    [TagColumn::Id, TagColumn::Name, TagColumn::CreatedAt];

/// Fields read into `CommentRow`.
pub(super) const COMMENT_COLUMNS: [CommentColumn; 6] = [
    CommentColumn::Id,
    CommentColumn::TaskId,
    CommentColumn::AuthorId,
    CommentColumn::Body,
    CommentColumn::CreatedAt,
    CommentColumn::EditedAt,
];

/// Fields read into `LinkRow`.
pub(super) const LINK_COLUMNS: [LinkColumn; 9] = [
    LinkColumn::Id,
    LinkColumn::TaskId,
    LinkColumn::Kind,
    LinkColumn::Label,
    LinkColumn::Url,
    LinkColumn::NoteId,
    LinkColumn::SortOrder,
    LinkColumn::CreatedAt,
    LinkColumn::UpdatedAt,
];
