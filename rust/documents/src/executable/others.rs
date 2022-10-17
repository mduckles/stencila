use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
};
use graph_triples::ResourceInfo;
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::{Address, Slot};
use node_dispatch::{dispatch_block, dispatch_inline, dispatch_node, dispatch_work};

use stencila_schema::*;

use crate::{
    executable::{AssembleContext, CompileContext, Executable, ExecuteContext},
    register_id,
};

// Nodes types that simply need an `id` assigned so that custom web component patch events have a target

macro_rules! executable_assemble_id_only {
    ($type:ty, $prefix:expr) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(
                &mut self,
                address: &mut Address,
                context: &mut AssembleContext,
            ) -> Result<()> {
                register_id!($prefix, self, address, context);
                Ok(())
            }
        }
    };
}

executable_assemble_id_only!(CodeBlock, "cb");
executable_assemble_id_only!(CodeFragment, "cf");
executable_assemble_id_only!(MathBlock, "mb");
executable_assemble_id_only!(MathFragment, "mf");

// Node types that do not need anything done

macro_rules! executable_nothing {
    ( $( $type:ty ),* ) => {
        $(
            impl Executable for $type {}
        )*
    };
}
executable_nothing!(
    // Primitives
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Date,
    Time,
    DateTime,
    Timestamp,
    Duration,
    Array,
    Object,
    // Entity types that are unlikely to ever need to be executable
    ThematicBreak,
    // Entity types that may need to be executable in the future
    Datatable,
    DatatableColumn,
    Periodical,
    PublicationIssue,
    PublicationVolume,
    Review,
    SoftwareApplication,
    Validator,
    ArrayValidator,
    BooleanValidator,
    ConstantValidator,
    DateTimeValidator,
    DateValidator,
    DurationValidator,
    EnumValidator,
    IntegerValidator,
    NumberValidator,
    StringValidator,
    TimestampValidator,
    TimeValidator,
    TupleValidator,
    // External resources
    File
);

// The following are "enum variant dispatching" implementations of `Executable` for
// the types that are also `Pointable`

#[async_trait]
impl Executable for Node {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        dispatch_node!(self, Box::pin(async { Ok(()) }), assemble, address, context).await
    }

    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        dispatch_node!(self, Box::pin(async { Ok(()) }), compile, context).await
    }

    async fn execute(&mut self, context: &mut ExecuteContext) -> Result<()> {
        dispatch_node!(self, Box::pin(async { Ok(()) }), execute, context).await
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        dispatch_node!(
            self,
            Box::pin(async { Ok(None) }),
            execute_begin,
            resource_info,
            kernel_space,
            kernel_selector,
            is_fork
        )
        .await
    }

    async fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
        dispatch_node!(
            self,
            Box::pin(async { Ok(()) }),
            execute_end,
            task_info,
            task_result
        )
        .await
    }
}

macro_rules! executable_enum {
    ($type: ty, $dispatch_macro: ident) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(
                &mut self,
                address: &mut Address,
                context: &mut AssembleContext,
            ) -> Result<()> {
                $dispatch_macro!(self, assemble, address, context).await
            }

            async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
                $dispatch_macro!(self, compile, context).await
            }

            async fn execute(&mut self, context: &mut ExecuteContext) -> Result<()> {
                $dispatch_macro!(self, execute, context).await
            }

            async fn execute_begin(
                &mut self,
                resource_info: &ResourceInfo,
                kernel_space: &KernelSpace,
                kernel_selector: &KernelSelector,
                is_fork: bool,
            ) -> Result<Option<TaskInfo>> {
                $dispatch_macro!(
                    self,
                    execute_begin,
                    resource_info,
                    kernel_space,
                    kernel_selector,
                    is_fork
                )
                .await
            }

            async fn execute_end(
                &mut self,
                task_info: TaskInfo,
                task_result: TaskResult,
            ) -> Result<()> {
                $dispatch_macro!(self, execute_end, task_info, task_result).await
            }
        }
    };
}

executable_enum!(CreativeWorkTypes, dispatch_work);
executable_enum!(BlockContent, dispatch_block);
executable_enum!(InlineContent, dispatch_inline);

/// Implementation of `Executable` for various fields of a struct
macro_rules! executable_fields {
    ($type:ty $(, $field:ident)* ) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(&mut self, address: &mut Address, context: &mut AssembleContext) -> Result<()> {
                $(
                    address.push_back(Slot::Name(stringify!($field).to_string()));
                    self.$field.assemble(address, context).await?;
                    address.pop_back();
                )*
                Ok(())
            }

            async fn execute(&mut self, context: &mut ExecuteContext) -> Result<()> {
                $(
                    self.$field.execute(context).await?;
                )*
                Ok(())
            }
        }
    };
}

executable_fields!(CiteGroup, items);

executable_fields!(Collection, parts);
executable_fields!(Directory, parts);

executable_fields!(List, items);
executable_fields!(ListItem, item, content);

executable_fields!(Table, rows, caption);
executable_fields!(TableSimple, rows, caption);
executable_fields!(TableRow, cells);
executable_fields!(TableCell, content);

/// Implementation of `Executable` for only the `content` field of a struct
macro_rules! executable_content {
    ($type:ty) => {
        executable_fields!($type, content);
    };
    ( $( $type:ty ),* ) => {
        $(
            executable_content!($type);
        )*
    };
}

executable_content!(
    Article,
    Cite,
    Claim,
    ClaimSimple,
    Comment,
    CreativeWork,
    Delete,
    Emphasis,
    Figure,
    FigureSimple,
    Heading,
    NontextualAnnotation,
    Note,
    Paragraph,
    Quote,
    QuoteBlock,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Underline
);

/// Implementation of `Executable` for enum variants
macro_rules! executable_variants {
    ( $type:ty $(, $variant:path )* ) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(&mut self, address: &mut Address, context: &mut AssembleContext) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.assemble(address, context).await,
                    )*
                }
            }
        }
    };
}

executable_variants!(
    CreativeWorkContent,
    CreativeWorkContent::String,
    CreativeWorkContent::VecNode
);

executable_variants!(
    DirectoryParts,
    DirectoryParts::File,
    DirectoryParts::Directory
);

executable_variants!(
    ListItemContent,
    ListItemContent::VecInlineContent,
    ListItemContent::VecBlockContent
);

executable_variants!(
    TableCaption,
    TableCaption::String,
    TableCaption::VecBlockContent
);

executable_variants!(
    TableCellContent,
    TableCellContent::VecInlineContent,
    TableCellContent::VecBlockContent
);
