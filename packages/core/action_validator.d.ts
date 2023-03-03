export type ParseErrorLocation = {
  index: number;
  line: number;
  column: number;
};

export type ValidationError =
  | {
      code: string;
      detail?: string;
      path: string;
      title: string;
      states?: Omit<ValidationState, "actionType">[];
    }
  | {
      code: string;
      detail: string;
      title: string;
      location?: ParseErrorLocation;
    };

export type ValidationState = {
  actionType: "action" | "workflow";
  errors: ValidationError[];
};

export function validateAction(src: string): ValidationState;
export function validateWorkflow(src: string): ValidationState;
