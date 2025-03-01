use crate::dml::validator::directive::{Args, DirectiveValidator, Error};
use crate::{ast, dml};

/// Prismas builtin `@embedded` directive.
pub struct EmbeddedDirectiveValidator {}

impl DirectiveValidator<dml::Model> for EmbeddedDirectiveValidator {
    fn directive_name(&self) -> &'static str {
        &"embedded"
    }
    fn validate_and_apply(&self, _args: &mut Args, obj: &mut dml::Model) -> Result<(), Error> {
        obj.is_embedded = true;
        Ok(())
    }

    fn serialize(&self, model: &dml::Model, _datamodel: &dml::Datamodel) -> Result<Option<ast::Directive>, Error> {
        if model.is_embedded {
            return Ok(Some(ast::Directive::new(self.directive_name(), vec![])));
        }

        Ok(None)
    }
}
