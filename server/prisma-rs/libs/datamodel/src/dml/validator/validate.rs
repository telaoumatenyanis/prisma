use super::common::*;
use crate::{
    ast, configuration, dml,
    errors::{ErrorCollection, ValidationError},
};

/// Helper for validating a datamodel.
///
/// When validating, we check if the datamodel is valid, and generate errors otherwise.
#[derive(Default)]
pub struct Validator {}

/// State error message. Seeing this error means something went really wrong internally. It's the datamodel equivalent of a bluescreen.
const STATE_ERROR: &str = "Failed lookup of model, field or optional property during internal processing. This means that the internal representation was mutated incorrectly.";

impl Validator {
    /// Creates a new instance, with all builtin directives registered.
    pub fn new() -> Validator {
        Self::default()
    }

    /// Creates a new instance, with all builtin directives and
    /// the directives defined by the given sources registered.
    ///
    /// The directives defined by the given sources will be namespaced.
    pub fn with_sources(_sources: &[Box<dyn configuration::Source>]) -> Validator {
        Self::default()
    }

    pub fn validate(&self, ast_schema: &ast::Datamodel, schema: &mut dml::Datamodel) -> Result<(), ErrorCollection> {
        let mut errors = ErrorCollection::new();

        // Model level validations.
        for model in schema.models() {
            if let Err(err) = self.validate_model_has_id(ast_schema.find_model(&model.name).expect(STATE_ERROR), model)
            {
                errors.push(err);
            }
            if let Err(err) = self.validate_id_fields_valid(ast_schema, model) {
                errors.push(err);
            }
            if let Err(err) = self.validate_relations_not_ambiguous(ast_schema, model) {
                errors.push(err);
            }
            if let Err(err) = self.validate_embedded_types_have_no_back_relation(ast_schema, schema, model) {
                errors.push(err);
            }
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn validate_model_has_id(&self, ast_model: &ast::Model, model: &dml::Model) -> Result<(), ValidationError> {
        if model.is_relation_model() {
            return Ok(());
            // Extempt from the id rule, we have an relation table.
        }

        match model.id_fields().count() {
            1 => Ok(()),
            _ => Err(ValidationError::new_model_validation_error(
                "Exactly one field must be marked as the id field with the `@id` directive.",
                &model.name,
                ast_model.span,
            )),
        }
    }

    fn validate_id_fields_valid(&self, ast_schema: &ast::Datamodel, model: &dml::Model) -> Result<(), ValidationError> {
        for id_field in model.id_fields() {
            let is_valid = match (&id_field.default_value, &id_field.field_type, &id_field.arity) {
                (
                    Some(dml::Value::Expression(name, return_type, args)),
                    dml::FieldType::Base(dml::ScalarType::String),
                    dml::FieldArity::Required,
                ) => {
                    let name_eq = name == "cuid" || name == "uuid";
                    let type_eq = return_type == &dml::ScalarType::String;
                    let args_eq = args.is_empty();

                    name_eq && type_eq && args_eq
                }
                (None, dml::FieldType::Base(dml::ScalarType::Int), dml::FieldArity::Required) => true,
                _ => false,
            };

            if !is_valid {
                return Err(ValidationError::new_model_validation_error(
                    "Invalid ID field. ID field must be one of: Int @id, String @id @default(cuid()), String @id @default(uuid()).", 
                    &model.name,
                    ast_schema.find_field(&model.name, &id_field.name).expect(STATE_ERROR).span));
            }
        }

        Ok(())
    }

    /// Ensures that embedded types do not have back relations
    /// to their parent types.
    fn validate_embedded_types_have_no_back_relation(
        &self,
        ast_schema: &ast::Datamodel,
        datamodel: &dml::Datamodel,
        model: &dml::Model,
    ) -> Result<(), ValidationError> {
        if model.is_embedded {
            for field in model.fields() {
                if !field.is_generated {
                    if let dml::FieldType::Relation(rel) = &field.field_type {
                        // TODO: I am not sure if this check is d'accord with the query engine.
                        let related = datamodel.find_model(&rel.to).unwrap();
                        let related_field = related.related_field(&model.name, &rel.name, &field.name).unwrap();

                        if rel.to_fields.is_empty() && !related_field.is_generated {
                            // TODO: Refactor that out, it's way too much boilerplate.
                            return Err(ValidationError::new_model_validation_error(
                                "Embedded models cannot have back relation fields.",
                                &model.name,
                                ast_schema.find_field(&model.name, &field.name).expect(STATE_ERROR).span,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Elegantly checks if any relations in the model are ambigious.
    fn validate_relations_not_ambiguous(
        &self,
        ast_schema: &ast::Datamodel,
        model: &dml::Model,
    ) -> Result<(), ValidationError> {
        for field_a in model.fields() {
            for field_b in model.fields() {
                if field_a != field_b {
                    if let dml::FieldType::Relation(rel_a) = &field_a.field_type {
                        if let dml::FieldType::Relation(rel_b) = &field_b.field_type {
                            if rel_a.to != model.name && rel_b.to != model.name {
                                // Not a self relation
                                // but pointing to the same foreign model,
                                // and also no names set.
                                if rel_a.to == rel_b.to && rel_a.name == rel_b.name {
                                    return Err(ValidationError::new_model_validation_error(
                                        "Ambiguous relation detected.",
                                        &model.name,
                                        ast_schema
                                            .find_field(&model.name, &field_a.name)
                                            .expect(STATE_ERROR)
                                            .span,
                                    ));
                                }
                            } else {
                                // A self relation...
                                for field_c in model.fields() {
                                    if field_a != field_c && field_b != field_c {
                                        if let dml::FieldType::Relation(rel_c) = &field_c.field_type {
                                            // ...but there are more thatn three fields without a name.
                                            if rel_c.to == model.name
                                                && rel_a.name == rel_b.name
                                                && rel_a.name == rel_c.name
                                            {
                                                return Err(ValidationError::new_model_validation_error(
                                                    "Ambiguous self relation detected.",
                                                    &model.name,
                                                    ast_schema
                                                        .find_field(&model.name, &field_a.name)
                                                        .expect(STATE_ERROR)
                                                        .span,
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
