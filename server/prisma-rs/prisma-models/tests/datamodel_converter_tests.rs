#![allow(non_snake_case)]
use prisma_models::*;
use std::sync::Arc;

#[test]
fn an_empty_datamodel_must_work() {
    let datamodel = convert("");
    assert_eq!(datamodel.enums.is_empty(), true);
    assert_eq!(datamodel.models().is_empty(), true);
    assert_eq!(datamodel.relations().is_empty(), true);
}

#[test]
fn converting_enums() {
    let datamodel = convert(
        r#"
            model MyModel {
                id Int @id
                field MyEnum
            }

            enum MyEnum {
                A
                B
                C
            }
        "#,
    );
    let expected_values = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let enm = datamodel.enums.iter().find(|e| e.name == "MyEnum").unwrap();
    assert_eq!(enm.values, vec!["A".to_string(), "B".to_string(), "C".to_string()]);

    let field = datamodel.assert_model("MyModel").assert_scalar_field("field");
    assert_eq!(field.type_identifier, TypeIdentifier::Enum);
    assert_eq!(
        field.internal_enum,
        Some(InternalEnum {
            name: "MyEnum".to_string(),
            values: expected_values
        })
    );
}

#[test]
fn models_with_only_scalar_fields() {
    let datamodel = convert(
        r#"
            model Test {
                id Int @id
                int Int
                float Float
                boolean Boolean
                dateTime DateTime
                stringOpt String?
                intList Int[]
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("id")
        .assert_type_identifier(TypeIdentifier::Int)
        .assert_behaviour(FieldBehaviour::Id {
            strategy: IdStrategy::Auto,
            sequence: None,
        })
        .assert_is_auto_generated_by_db();
    model
        .assert_scalar_field("int")
        .assert_type_identifier(TypeIdentifier::Int)
        .assert_no_behaviour();
    model
        .assert_scalar_field("float")
        .assert_type_identifier(TypeIdentifier::Float)
        .assert_no_behaviour();
    model
        .assert_scalar_field("boolean")
        .assert_type_identifier(TypeIdentifier::Boolean)
        .assert_no_behaviour();
    model
        .assert_scalar_field("dateTime")
        .assert_type_identifier(TypeIdentifier::DateTime)
        .assert_no_behaviour();
    model
        .assert_scalar_field("stringOpt")
        .assert_type_identifier(TypeIdentifier::String)
        .assert_optional()
        .assert_no_behaviour();
    model
        .assert_scalar_field("intList")
        .assert_type_identifier(TypeIdentifier::Int)
        .assert_list();
}

#[test]
fn db_names_work() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(cuid())
                field String @map(name:"my_column")
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    let field = model.assert_scalar_field("field");
    assert_eq!(
        field.manifestation,
        Some(FieldManifestation {
            db_name: "my_column".to_string()
        })
    )
}

#[test]
#[ignore]
fn scalar_lists_work() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(cuid())
                intList Int[]
            }
        "#,
    );
    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("intList")
        .assert_type_identifier(TypeIdentifier::Int)
        .assert_list()
        .assert_behaviour(FieldBehaviour::ScalarList {
            strategy: ScalarListStrategy::Relation,
        });
}

#[test]
fn unique_works() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(cuid())
                unique String @unique
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("unique")
        .assert_type_identifier(TypeIdentifier::String)
        .assert_unique();
}

#[test]
fn uuid_fields_must_work() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(uuid())
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("id")
        .assert_type_identifier(TypeIdentifier::UUID);
}

#[test]
fn cuid_fields_must_work() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(cuid())
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("id")
        .assert_type_identifier(TypeIdentifier::GraphQLID);
}

#[test]
fn createdAt_works() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(cuid())
                createdAt DateTime @default(now())
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("createdAt")
        .assert_type_identifier(TypeIdentifier::DateTime)
        .assert_created_at();
}

#[test]
fn updatedAt_works() {
    let datamodel = convert(
        r#"
            model Test {
                id String @id @default(cuid())
                updatedAt DateTime @updatedAt
            }
        "#,
    );

    let model = datamodel.assert_model("Test");
    model
        .assert_scalar_field("updatedAt")
        .assert_type_identifier(TypeIdentifier::DateTime)
        .assert_updated_at();
}

#[test]
fn explicit_relation_fields() {
    let datamodel = convert(
        r#"
            model Blog {
                id Int @id
                posts Post[]
            }

            model Post {
                id Int @id
                blog Blog? @map(name:"blog_id")
            }
        "#,
    );

    let relation_name = "BlogToPost";
    let blog = datamodel.assert_model("Blog");
    let post = datamodel.assert_model("Post");
    let relation = datamodel.assert_relation(relation_name);

    blog.assert_relation_field("posts")
        .assert_list()
        .assert_optional()
        .assert_relation_name(relation_name)
        .assert_side(RelationSide::A);

    post.assert_relation_field("blog")
        .assert_optional()
        .assert_relation_name(relation_name)
        .assert_side(RelationSide::B);

    relation
        .assert_name(relation_name)
        .assert_model_a("Blog")
        .assert_model_b("Post")
        .assert_manifestation(RelationLinkManifestation::Inline(InlineRelation {
            in_table_of_model_name: "Post".to_string(),
            referencing_column: "blog_id".to_string(),
        }));
}

#[test]
fn many_to_many_relations() {
    let datamodel = convert(
        r#"
            model Post {
                id Int @id
                blogs Blog[]
            }

            model Blog {
                id Int @id
                posts Post[]
            }
        "#,
    );

    let relation_name = "BlogToPost";
    let blog = datamodel.assert_model("Blog");
    let post = datamodel.assert_model("Post");
    let relation = datamodel.assert_relation(relation_name);

    blog.assert_relation_field("posts")
        .assert_list()
        .assert_optional()
        .assert_relation_name(relation_name)
        .assert_side(RelationSide::A);

    post.assert_relation_field("blogs")
        .assert_list()
        .assert_optional()
        .assert_relation_name(relation_name)
        .assert_side(RelationSide::B);

    relation
        .assert_name(relation_name)
        .assert_model_a("Blog")
        .assert_model_b("Post")
        .assert_manifestation(RelationLinkManifestation::RelationTable(RelationTable {
            table: format!("_{}", relation_name),
            model_a_column: "A".to_string(),
            model_b_column: "B".to_string(),
            id_column: None,
        }));
}

#[test]
fn implicit_relation_fields() {
    let datamodel = convert(
        r#"
            model Blog {
                id Int @id
                posts Post[]
            }

            model Post {
                id Int @id
            }
        "#,
    );

    let relation_name = "BlogToPost";
    let post = datamodel.assert_model("Post");
    let relation = datamodel.assert_relation(relation_name);

    post.assert_relation_field("blog").assert_optional();

    relation
        .assert_name(relation_name)
        .assert_model_a("Blog")
        .assert_model_b("Post")
        .assert_manifestation(RelationLinkManifestation::Inline(InlineRelation {
            in_table_of_model_name: "Post".to_string(),
            referencing_column: "blog".to_string(),
        }));
}

#[test]
fn explicit_relation_names() {
    let datamodel = convert(
        r#"
            model Blog {
                id Int @id
                posts Post[] @relation(name: "MyRelationName")
            }

            model Post {
                id Int @id
                blog Blog? @relation(name: "MyRelationName")
            }
        "#,
    );

    let blog = datamodel.assert_model("Blog");
    let post = datamodel.assert_model("Post");

    let relation_name = "MyRelationName";
    blog.assert_relation_field("posts")
        .assert_list()
        .assert_optional()
        .assert_relation_name(relation_name);
    post.assert_relation_field("blog")
        .assert_optional()
        .assert_relation_name(relation_name);
}

#[test]
#[ignore]
fn self_relations() {
    let datamodel = convert(
        r#"
            model Employee {
                id Int @id
                ReportsTo: Employee?
            }
        "#,
    );

    let employee = datamodel.assert_model("Employee");

    employee
        .assert_relation_field("ReportsTo")
        .assert_relation_name("EmployeeToEmployee");
    // employee.assert_relation_field("employee");
}

#[test]
fn ambiguous_relations() {
    let datamodel = convert(
        r#"
            model Blog {
                id    Int  @id
                post1 Post @relation(name: "Relation1")
                post2 Post @relation(name: "Relation2")
            }
            
            model Post {
                id    Int  @id
                blog1 Blog @relation(name: "Relation1")
                blog2 Blog @relation(name: "Relation2")
            }
        "#,
    );

    let blog = datamodel.assert_model("Blog");
    blog.assert_relation_field("post1").assert_relation_name("Relation1");
    blog.assert_relation_field("post2").assert_relation_name("Relation2");

    let post = datamodel.assert_model("Post");
    post.assert_relation_field("blog1").assert_relation_name("Relation1");
    post.assert_relation_field("blog2").assert_relation_name("Relation2");
}

fn convert(datamodel: &str) -> Arc<InternalDataModel> {
    let datamodel = dbg!(datamodel::parse(datamodel).unwrap());
    let template = DatamodelConverter::convert(&datamodel);
    template.build("not_important".to_string())
}

trait DatamodelAssertions {
    fn assert_model(&self, name: &str) -> Arc<Model>;
    fn assert_relation(&self, name: &str) -> Arc<Relation>;
}

impl DatamodelAssertions for InternalDataModel {
    fn assert_model(&self, name: &str) -> Arc<Model> {
        self.find_model(name).unwrap()
    }

    fn assert_relation(&self, name: &str) -> Arc<Relation> {
        self.find_relation(name).unwrap().upgrade().unwrap()
    }
}

trait ModelAssertions {
    fn assert_scalar_field(&self, name: &str) -> Arc<ScalarField>;
    fn assert_relation_field(&self, name: &str) -> Arc<RelationField>;
}

impl ModelAssertions for Model {
    fn assert_scalar_field(&self, name: &str) -> Arc<ScalarField> {
        self.fields().find_from_scalar(name).unwrap()
    }

    fn assert_relation_field(&self, name: &str) -> Arc<RelationField> {
        self.fields().find_from_relation_fields(name).unwrap()
    }
}

trait FieldAssertions {
    fn assert_type_identifier(&self, ti: TypeIdentifier) -> &Self;
    fn assert_optional(&self) -> &Self;
    fn assert_list(&self) -> &Self;
    fn assert_unique(&self) -> &Self;
}

trait ScalarFieldAssertions {
    fn assert_updated_at(&self) -> &Self;
    fn assert_created_at(&self) -> &Self;
    fn assert_behaviour(&self, behaviour: FieldBehaviour) -> &Self;
    fn assert_no_behaviour(&self) -> &Self;
    fn assert_is_auto_generated_by_db(&self) -> &Self;
}

trait RelationFieldAssertions {
    fn assert_relation_name(&self, name: &str) -> &Self;
    fn assert_side(&self, side: RelationSide) -> &Self;
}

impl FieldAssertions for ScalarField {
    fn assert_type_identifier(&self, ti: TypeIdentifier) -> &Self {
        assert_eq!(self.type_identifier, ti);
        self
    }

    fn assert_optional(&self) -> &Self {
        assert!(!self.is_required);
        self
    }

    fn assert_list(&self) -> &Self {
        assert!(self.is_list);
        self
    }

    fn assert_unique(&self) -> &Self {
        assert!(self.is_unique());
        self
    }
}

impl ScalarFieldAssertions for ScalarField {
    fn assert_created_at(&self) -> &Self {
        self.assert_behaviour(FieldBehaviour::CreatedAt);
        self
    }

    fn assert_updated_at(&self) -> &Self {
        self.assert_behaviour(FieldBehaviour::UpdatedAt);
        self
    }

    fn assert_behaviour(&self, behaviour: FieldBehaviour) -> &Self {
        assert_eq!(self.behaviour, Some(behaviour));
        self
    }

    fn assert_no_behaviour(&self) -> &Self {
        assert!(self.behaviour.is_none());
        self
    }

    fn assert_is_auto_generated_by_db(&self) -> &Self {
        assert!(self.is_auto_generated);
        self
    }
}

impl FieldAssertions for RelationField {
    fn assert_type_identifier(&self, ti: TypeIdentifier) -> &Self {
        assert_eq!(self.type_identifier, ti);
        self
    }

    fn assert_optional(&self) -> &Self {
        assert!(!self.is_required);
        self
    }

    fn assert_list(&self) -> &Self {
        assert!(self.is_list);
        self
    }

    fn assert_unique(&self) -> &Self {
        assert!(self.is_unique());
        self
    }
}

impl RelationFieldAssertions for RelationField {
    fn assert_relation_name(&self, name: &str) -> &Self {
        assert_eq!(self.relation_name, name);
        self
    }

    fn assert_side(&self, side: RelationSide) -> &Self {
        assert_eq!(self.relation_side, side);
        self
    }
}

trait RelationAssertions {
    fn assert_name(&self, name: &str) -> &Self;
    fn assert_model_a(&self, name: &str) -> &Self;
    fn assert_model_b(&self, name: &str) -> &Self;
    fn assert_manifestation(&self, mani: RelationLinkManifestation) -> &Self;
}

impl RelationAssertions for Relation {
    fn assert_name(&self, name: &str) -> &Self {
        assert_eq!(self.name, name);
        self
    }
    fn assert_model_a(&self, name: &str) -> &Self {
        assert_eq!(self.model_a().name, name);
        self
    }
    fn assert_model_b(&self, name: &str) -> &Self {
        assert_eq!(self.model_b().name, name);
        self
    }
    fn assert_manifestation(&self, manifestation: RelationLinkManifestation) -> &Self {
        assert_eq!(self.manifestation, Some(manifestation));
        self
    }
}
