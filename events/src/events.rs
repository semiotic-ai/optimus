
use substreams::Hex;
use substreams_database_change::pb::database::TableChange;

pub trait ToTableChange {
    fn add_table_changes(&self, table_change: &mut TableChange);

    fn get_table_name(&self) -> &'static str;

    fn get_contract_name(&self) -> &'static str;
}

pub trait TableField {
    fn get_value(&self) -> String;
}

impl TableField for Vec<u8> {
    fn get_value(&self) -> String {
        Hex(&self).to_string()
    }
}

impl TableField for substreams::scalar::BigInt {
    fn get_value(&self) -> String {
        self.to_string()
    }
}

impl<T:TableField> TableField for Vec<T> {
    fn get_value(&self) -> String {
        format!("[{}]", self.iter().map(|f| f.get_value()).collect::<Vec<_>>().join(","))
    }
}

impl<T1:TableField,T2:TableField> TableField for (T1,T2) {
    fn get_value(&self) -> String {
        format!(
            "({},{})", 
            self.0.get_value(),
            self.1.get_value()
        )
    }
}

impl<T1:TableField,T2:TableField,T3:TableField> TableField for (T1,T2,T3) {
    fn get_value(&self) -> String {
        format!(
            "({},{},{})", 
            self.0.get_value(),
            self.1.get_value(),
            self.2.get_value()
        )
    }
}

impl<T1:TableField,T2:TableField,T3:TableField,T4:TableField> TableField for (T1,T2,T3,T4) {
    fn get_value(&self) -> String {
        format!(
            "({},{},{},{})", 
            self.0.get_value(),
            self.1.get_value(),
            self.2.get_value(),
            self.3.get_value()
        )
    }
}

impl<T1:TableField,T2:TableField,T3:TableField,T4:TableField,T5:TableField> TableField for (T1,T2,T3,T4,T5) {
    fn get_value(&self) -> String {
        format!(
            "({},{},{},{},{})", 
            self.0.get_value(),
            self.1.get_value(),
            self.2.get_value(),
            self.3.get_value(),
            self.4.get_value()
        )
    }
}




