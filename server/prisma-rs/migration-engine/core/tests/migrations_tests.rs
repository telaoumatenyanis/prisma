#![allow(non_snake_case)]
#![allow(unused)]
mod test_harness;
use sql_migration_connector::database_inspector::*;
use sql_migration_connector::SqlFamily;
use test_harness::*;

#[test]
fn adding_a_scalar_field_must_work() {
    test_each_connector(|_, api| {
        let dm2 = r#"
            model Test {
                id String @id @default(cuid())
                int Int
                float Float
                boolean Boolean
                string String
                dateTime DateTime
                enum MyEnum
            }

            enum MyEnum {
                A
                B
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let table = result.table_bang("Test");
        table.columns.iter().for_each(|c| assert_eq!(c.is_required, true));

        assert_eq!(table.column_bang("int").tpe, ColumnType::Int);
        assert_eq!(table.column_bang("float").tpe, ColumnType::Float);
        assert_eq!(table.column_bang("boolean").tpe, ColumnType::Boolean);
        assert_eq!(table.column_bang("string").tpe, ColumnType::String);
        assert_eq!(table.column_bang("dateTime").tpe, ColumnType::DateTime);
        assert_eq!(table.column_bang("enum").tpe, ColumnType::String);
    });
}

//#[test]
//fn apply_schema() {
//    test_each_connector(|api| {
//        let dm2 = r#"
//            model Test {
//                id String @id @default(cuid())
//                int Int
//                float Float
//                boolean Boolean
//                string String
//                dateTime DateTime
//                enum MyEnum
//            }
//
//            enum MyEnum {
//                A
//                B
//            }
//        "#;
//
//        infer_and_apply(api, &dm2);
//    });
//}

#[test]
fn adding_an_optional_field_must_work() {
    test_each_connector(|_, api| {
        let dm2 = r#"
            model Test {
                id String @id @default(cuid())
                field String?
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("Test").column_bang("field");
        assert_eq!(column.is_required, false);
    });
}

#[test]
fn adding_an_id_field_with_a_special_name_must_work() {
    test_each_connector(|_, api| {
        let dm2 = r#"
            model Test {
                specialName String @id @default(cuid())
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("Test").column("specialName");
        assert_eq!(column.is_some(), true);
    });
}

#[test]
fn removing_a_scalar_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model Test {
                id String @id @default(cuid())
                field String
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column1 = result.table_bang("Test").column("field");
        assert_eq!(column1.is_some(), true);

        let dm2 = r#"
            model Test {
                id String @id @default(cuid())
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column2 = result.table_bang("Test").column("field");
        assert_eq!(column2.is_some(), false);
    });
}

#[test]
fn can_handle_reserved_sql_keywords_for_model_name() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model Group {
                id String @id @default(cuid())
                field String
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column = result.table_bang("Group").column_bang("field");
        assert_eq!(column.tpe, ColumnType::String);

        let dm2 = r#"
            model Group {
                id String @id @default(cuid())
                field Int
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("Group").column_bang("field");
        assert_eq!(column.tpe, ColumnType::Int);
    });
}

#[test]
fn can_handle_reserved_sql_keywords_for_field_name() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model Test {
                id String @id @default(cuid())
                Group String
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column = result.table_bang("Test").column_bang("Group");
        assert_eq!(column.tpe, ColumnType::String);

        let dm2 = r#"
            model Test {
                id String @id @default(cuid())
                Group Int
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("Test").column_bang("Group");
        assert_eq!(column.tpe, ColumnType::Int);
    });
}

#[test]
fn update_type_of_scalar_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model Test {
                id String @id @default(cuid())
                field String
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column1 = result.table_bang("Test").column_bang("field");
        assert_eq!(column1.tpe, ColumnType::String);

        let dm2 = r#"
            model Test {
                id String @id @default(cuid())
                field Int
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column2 = result.table_bang("Test").column_bang("field");
        assert_eq!(column2.tpe, ColumnType::Int);
    });
}

#[test]
fn changing_the_type_of_an_id_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b  B   @relation(references: [id])
            }
            model B {
                id Int @id
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.tpe, ColumnType::Int);
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );

        let dm2 = r#"
            model A {
                id Int @id
                b  B   @relation(references: [id])
            }
            model B {
                id String @id @default(cuid())
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.tpe, ColumnType::String);
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
fn updating_db_name_of_a_scalar_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id String @id @default(cuid())
                field String @map(name:"name1")
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        assert_eq!(result.table_bang("A").column("name1").is_some(), true);

        let dm2 = r#"
            model A {
                id String @id @default(cuid())
                field String @map(name:"name2")
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        assert_eq!(result.table_bang("A").column("name1").is_some(), false);
        assert_eq!(result.table_bang("A").column("name2").is_some(), true);
    });
}

#[test]
fn changing_a_relation_field_to_a_scalar_field_must_work() {
    // this relies on link: INLINE which we don't support yet
    test_each_connector_with_ignores(vec![SqlFamily::Mysql], |_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b B @relation(references: [id])
            }
            model B {
                id Int @id
                a A // remove this once the implicit back relation field is implemented
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.foreign_key.is_some(), true);
        assert_eq!(column.tpe, ColumnType::Int);

        let dm2 = r#"
            model A {
                id Int @id
                b String
            }
            model B {
                id Int @id
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.foreign_key.is_some(), false);
        assert_eq!(column.tpe, ColumnType::String);
    });
}

#[test]
fn changing_a_scalar_field_to_a_relation_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b String
            }
            model B {
                id Int @id
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.foreign_key.is_some(), false);
        assert_eq!(column.tpe, ColumnType::String);

        let dm2 = r#"
            model A {
                id Int @id
                b B @relation(references: [id])
            }
            model B {
                id Int @id
                a A // remove this once the implicit back relation field is implemented
            }
        "#;
        let result = infer_and_apply(api, &dm2);
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.foreign_key.is_some(), true);
        assert_eq!(column.tpe, ColumnType::Int);
    });
}

#[test]
fn adding_a_many_to_many_relation_must_result_in_a_prisma_style_relation_table() {
    // TODO: one model should have an id of different type. Not possible right now due to barrel limitation.
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                bs B[]
            }
            model B {
                id Int @id
                as A[]
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let relation_table = result.table_bang("_AToB");
        assert_eq!(relation_table.columns.len(), 2);
        let aColumn = relation_table.column_bang("A");
        assert_eq!(aColumn.tpe, ColumnType::Int);
        assert_eq!(
            aColumn.foreign_key,
            Some(ForeignKey::new("A".to_string(), "id".to_string(), OnDelete::NoAction))
        );
        let bColumn = relation_table.column_bang("B");
        assert_eq!(bColumn.tpe, ColumnType::Int);
        assert_eq!(
            bColumn.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
fn adding_a_many_to_many_relation_with_custom_name_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                bs B[] @relation(name: "my_relation")
            }
            model B {
                id Int @id
                as A[] @relation(name: "my_relation")
            }
        "#;

        let result = infer_and_apply(api, &dm1);
        let relation_table = result.table_bang("_my_relation");
        assert_eq!(relation_table.columns.len(), 2);

        let aColumn = relation_table.column_bang("A");
        assert_eq!(aColumn.tpe, ColumnType::Int);
        assert_eq!(
            aColumn.foreign_key,
            Some(ForeignKey::new("A".to_string(), "id".to_string(), OnDelete::NoAction))
        );

        let bColumn = relation_table.column_bang("B");
        assert_eq!(bColumn.tpe, ColumnType::Int);
        assert_eq!(
            bColumn.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
#[ignore]
fn adding_a_many_to_many_relation_for_exotic_id_types_must_work() {
    // TODO: add this once we have figured out what id types we support
    unimplemented!();
}

#[test]
#[ignore]
fn forcing_a_relation_table_for_a_one_to_many_relation_must_work() {
    // TODO: implement this once we have decided if this is actually possible in dm v2
    unimplemented!();
}

// #[test]
// #[ignore]
// fn forcing_a_relation_table_for_a_one_to_many_relation_must_work() {
//     // TODO: implement this once we have decided if this is actually possible in dm v2
//     unimplemented!();
// }

#[test]
#[ignore]
fn providing_an_explicit_link_table_must_work() {
    // TODO: implement this once we have decided if this is actually possible in dm v2
    unimplemented!();
}

#[test]
fn adding_an_inline_relation_must_result_in_a_foreign_key_in_the_model_table() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b B @relation(references: [id])
            }

            model B {
                id Int @id
                a A // todo: remove when implicit back relation field is implemented
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm1));
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.tpe, ColumnType::Int);
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
fn specifying_a_db_name_for_an_inline_relation_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b B @relation(references: [id]) @map(name: "b_column")
            }

            model B {
                id Int @id
                a A // todo: remove when implicit back relation field is implemented
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm1));
        let column = result.table_bang("A").column_bang("b_column");
        assert_eq!(column.tpe, ColumnType::Int);
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
fn adding_an_inline_relation_to_a_model_with_an_exotic_id_type() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b B @relation(references: [id])
            }

            model B {
                id String @id @default(cuid())
                a A // todo: remove when implicit back relation field is implemented
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm1));
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(column.tpe, ColumnType::String);
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
fn removing_an_inline_relation_must_work() {
    test_each_connector_with_ignores(vec![SqlFamily::Mysql], |_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b B @relation(references: [id])
            }

            model B {
                id Int @id
                a A // todo: remove when implicit back relation field is implemented
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm1));
        let column = result.table_bang("A").column("b");
        assert_eq!(column.is_some(), true);

        let dm2 = r#"
            model A {
                id Int @id
            }

            model B {
                id Int @id
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm2));
        let column = result.table_bang("A").column("b");
        assert_eq!(column.is_some(), false);
    });
}

#[test]
fn moving_an_inline_relation_to_the_other_side_must_work() {
    test_each_connector_with_ignores(vec![SqlFamily::Mysql], |_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                b B @relation(references: [id])
            }

            model B {
                id Int @id
                a A // todo: remove when implicit back relation field is implemented
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm1));
        let column = result.table_bang("A").column_bang("b");
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("B".to_string(), "id".to_string(), OnDelete::NoAction))
        );

        let dm2 = r#"
            model A {
                id Int @id
                b B // todo: remove when implicit back relation field is implemented
            }

            model B {
                id Int @id
                a A @relation(references: [id])
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm2));
        let column = result.table_bang("B").column_bang("a");
        assert_eq!(
            column.foreign_key,
            Some(ForeignKey::new("A".to_string(), "id".to_string(), OnDelete::NoAction))
        );
    });
}

#[test]
fn adding_a_new_unique_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                field String @unique
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm1));
        let index = result
            .table_bang("A")
            .indexes
            .iter()
            .find(|i| i.columns == vec!["field"]);
        // FIXME: bring assertion back once introspection can handle indexes
        //        assert_eq!(index.is_some(), true);
        //        assert_eq!(index.unwrap().tpe, IndexType::Unique);
    });
}

#[test]
fn removing_an_existing_unique_field_must_work() {
    //    test_only_connector(SqlFamily::Postgres, |_, api| {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id    Int    @id
                field String @unique
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        // FIXME: bring assertion back once introspection can handle indexes
        //        let index = result
        //            .table_bang("A")
        //            .indexes
        //            .iter()
        //            .find(|i| i.columns == vec!["field"]);
        //        assert_eq!(index.is_some(), true);
        //        assert_eq!(index.unwrap().tpe, IndexType::Unique);

        let dm2 = r#"
            model A {
                id    Int    @id
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm2));
        // FIXME: bring assertion back once introspection can handle indexes
        //        let index = result
        //            .table_bang("A")
        //            .indexes
        //            .iter()
        //            .find(|i| i.columns == vec!["field"]);
        //        assert_eq!(index.is_some(), false);
    });
}

#[test]
fn adding_unique_to_an_existing_field_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id    Int    @id
                field String
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        // FIXME: bring assertion back once introspection can handle indexes
        //        let index = result
        //            .table_bang("A")
        //            .indexes
        //            .iter()
        //            .find(|i| i.columns == vec!["field"]);
        //        assert_eq!(index.is_some(), true);
        //        assert_eq!(index.unwrap().tpe, IndexType::Unique);

        let dm2 = r#"
            model A {
                id    Int    @id
                field String @unique
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm2));
        // FIXME: bring assertion back once introspection can handle indexes
        //        let index = result
        //            .table_bang("A")
        //            .indexes
        //            .iter()
        //            .find(|i| i.columns == vec!["field"]);
        //        assert_eq!(index.is_some(), false);
    });
}

#[test]
fn removing_unique_from_an_existing_field_must_work() {
    //    test_only_connector(SqlFamily::Postgres, |_, api| {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id    Int    @id
                field String @unique
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        // FIXME: bring assertion back once introspection can handle indexes
        //        let index = result
        //            .table_bang("A")
        //            .indexes
        //            .iter()
        //            .find(|i| i.columns == vec!["field"]);
        //        assert_eq!(index.is_some(), true);
        //        assert_eq!(index.unwrap().tpe, IndexType::Unique);

        let dm2 = r#"
            model A {
                id    Int    @id
                field String
            }
        "#;
        let result = dbg!(infer_and_apply(api, &dm2));
        // FIXME: bring assertion back once introspection can handle indexes
        //        let index = result
        //            .table_bang("A")
        //            .indexes
        //            .iter()
        //            .find(|i| i.columns == vec!["field"]);
        //        assert_eq!(index.is_some(), false);
    });
}

#[test]
fn adding_a_scalar_list_for_a_modelwith_id_type_int_must_work() {
    test_each_connector(|_, api| {
        let dm1 = r#"
            model A {
                id Int @id
                strings String[]
                enums Status[]
            }
            
            enum Status {
              OK
              ERROR
            }
        "#;
        let result = infer_and_apply(api, &dm1);
        let scalar_list_table_for_strings = result.table_bang("A_strings");
        let node_id_column = scalar_list_table_for_strings.column_bang("nodeId");
        assert_eq!(node_id_column.tpe, ColumnType::Int);
        assert_eq!(
            scalar_list_table_for_strings.primary_key_columns,
            vec!["nodeId", "position"]
        );
        let scalar_list_table_for_enums = result.table_bang("A_enums");
        let node_id_column = scalar_list_table_for_enums.column_bang("nodeId");
        assert_eq!(node_id_column.tpe, ColumnType::Int);
        assert_eq!(
            scalar_list_table_for_enums.primary_key_columns,
            vec!["nodeId", "position"]
        );
    });
}

#[test]
fn updating_a_model_with_a_scalar_list_to_a_different_id_type_must_work() {
    test_each_connector_with_ignores(vec![SqlFamily::Mysql], |_, api| {
        let dm = r#"
            model A {
                id Int @id
                strings String[]
            }
        "#;
        let result = infer_and_apply(api, &dm);
        let node_id_column = result.table_bang("A_strings").column_bang("nodeId");
        assert_eq!(node_id_column.tpe, ColumnType::Int);

        let dm = r#"
            model A {
                id String @id @default(cuid())
                strings String[]
            }
        "#;
        let result = infer_and_apply(api, &dm);
        let node_id_column = result.table_bang("A_strings").column_bang("nodeId");
        assert_eq!(node_id_column.tpe, ColumnType::String);
    });
}

#[test]
fn reserved_sql_key_words_must_work() {
    // Group is a reserved keyword
    test_each_connector(|_, api| {
        let dm = r#"
            model Group {
                id    String  @default(cuid()) @id
                parent Group? @relation(name: "ChildGroups")
                childGroups Group[] @relation(name: "ChildGroups")
            }
        "#;
        let result = infer_and_apply(api, &dm);

        let relation_column = result.table_bang("Group").column_bang("parent");
        assert_eq!(
            relation_column.foreign_key,
            Some(ForeignKey {
                name: None,
                table: "Group".to_string(),
                column: "id".to_string(),
                on_delete: OnDelete::NoAction,
            })
        )
    });
}
