// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeError } from './CodeError';
import { Duration } from './Duration';
import { ExecutionAuto } from './ExecutionAuto';
import { ExecutionDependant } from './ExecutionDependant';
import { ExecutionDependency } from './ExecutionDependency';
import { ExecutionDigest } from './ExecutionDigest';
import { ExecutionRequired } from './ExecutionRequired';
import { ExecutionStatus } from './ExecutionStatus';
import { ExecutionTag } from './ExecutionTag';
import { Integer } from './Integer';
import { Node } from './Node';
import { Timestamp } from './Timestamp';
import { Validator } from './Validator';

// A parameter of a document.
export class Parameter {
  type = "Parameter";

  // The identifier for this item
  id?: string;

  // Under which circumstances the code should be automatically executed.
  executionAuto?: ExecutionAuto;

  // A digest of the content, semantics and dependencies of the node.
  compilationDigest?: ExecutionDigest;

  // The `compileDigest` of the node when it was last executed.
  executionDigest?: ExecutionDigest;

  // The upstream dependencies of this node.
  executionDependencies?: ExecutionDependency[];

  // The downstream dependants of this node.
  executionDependants?: ExecutionDependant[];

  // Tags in the code which affect its execution
  executionTags?: ExecutionTag[];

  // A count of the number of times that the node has been executed.
  executionCount?: Integer;

  // Whether, and why, the code requires execution or re-execution.
  executionRequired?: ExecutionRequired;

  // The id of the kernel that the node was last executed in.
  executionKernel?: string;

  // Status of the most recent, including any current, execution.
  executionStatus?: ExecutionStatus;

  // The timestamp when the last execution ended.
  executionEnded?: Timestamp;

  // Duration of the last execution.
  executionDuration?: Duration;

  // Errors when compiling (e.g. syntax errors) or executing the node.
  errors?: CodeError[];

  // The name of the parameter.
  name: string;

  // A short label for the parameter.
  label?: string;

  // The current value of the parameter.
  value?: Node;

  // The default value of the parameter.
  default?: Node;

  // The validator that the value is validated against.
  validator?: Validator;

  // Whether the parameter should be hidden.
  hidden?: boolean;

  // The dotted path to the object (e.g. a database table column) that the parameter should be derived from
  derivedFrom?: string;

  constructor(name: string, options?: Parameter) {
    if (options) Object.assign(this, options)
    this.name = name;
  }
}
