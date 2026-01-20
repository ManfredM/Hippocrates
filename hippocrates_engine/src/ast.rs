use crate::domain::{Unit, ValueType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Definition {
    Value(ValueDef),
    Period(PeriodDef),
    Plan(PlanDef),
    Drug(DrugDef),
    Addressee(AddresseeDef),
    Context(ContextDef),
    Unit(UnitDef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDef {
    pub name: String,
    pub plurals: Vec<String>,
    pub singulars: Vec<String>,
    pub abbreviations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueDef {
    pub name: String,
    pub value_type: ValueType,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugDef {
    pub name: String,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddresseeDef {
    pub name: String,
    pub is_group: bool,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDef {
    pub items: Vec<ContextItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodDef {
    pub name: String,
    pub timeframes: Vec<RangeSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDef {
    pub name: String,
    pub blocks: Vec<PlanBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanBlock {
    DuringPlan(Vec<Statement>),
    Event(EventBlock),
    Trigger(TriggerBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBlock {
    pub name: String,
    pub trigger: Trigger,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerBlock {
    pub trigger: Trigger,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    Periodic {
        interval: f64,
        interval_unit: Unit,
        duration: Option<(f64, Unit)>,
    },
    StartOf(String),
    ChangeOf(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Property {
    ValidValues(Vec<Statement>),
    Meaning(Vec<AssessmentCase>),
    Question(Action), // AskQuestion
    Calculation(Vec<Statement>),
    Reuse(f64, Unit),
    Documentation(String),
    Inheritance(String, Option<Vec<Property>>), // inherit from identifier
    Ingredients(Vec<Ingredient>),
    DosageSafety(Vec<DosageRule>),
    Administration(Vec<AdminRule>),
    Interactions(Vec<InteractionRule>),
    ContactInfo(Vec<ContactDetail>),
    AfterConsentRejected(Vec<Statement>),
    GroupedAddressees(Vec<String>),
    ContactOrder(String), // Parallel or Sequence
    Timeframe(Vec<Vec<RangeSelector>>),
    Custom(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub amount: f64,
    pub unit: Unit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DosageRule {
    MaxSingle(Expression),
    MaxDaily(Expression),
    MinTimeBetween(Expression), // Period
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminRule {
    Form(String),
    Schedule(String, Expression, String), // drug period after drug
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRule {
    pub drug: String,
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactDetail {
    Email(String),
    Phone(String),
    HippocratesId(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextItem {
    Timeframe(RangeSelector),
    Data(String),
    ValueFilter(AssessmentCase),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatementKind {
    Assignment(Assignment),
    Action(Action),
    Conditional(Conditional),
    ContextBlock(ContextBlock),
    EventProgression(String, Vec<AssessmentCase>),
    Command(String),
    Constraint(Expression, String, RangeSelector),
    NoOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub target: String,
    pub expression: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum Action {
    ShowMessage(Vec<Expression>, Option<Vec<Statement>>),
    AskQuestion(String, Option<Vec<Statement>>),
    SendInfo(String, Vec<Expression>),
    ListenFor(String),
    StartPeriod,
    Configure(String),
    MessageExpiration(RangeSelector),
    ValidateAnswer(crate::domain::ValidationMode, Option<(f64, Unit)>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditional {
    pub condition: ConditionalTarget, // Changed from Expression
    pub cases: Vec<AssessmentCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionalTarget {
    Expression(Expression),
    Confidence(String), // identifier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub items: Vec<ContextItem>, // Changed from context_query
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentCase {
    pub condition: RangeSelector,
    pub block: Block,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeSelector {
    Between(Expression, Expression),
    Equals(Expression),
    GreaterThan(Expression),
    List(Vec<Expression>),
    Range(Expression, Expression),
    Comparison(Expression, ConditionOperator, Expression),
    Condition(ConditionOperator, Expression),
    NotEnoughData,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
}

pub type Block = Vec<Statement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Binary(Box<Expression>, String, Box<Expression>),
    Statistical(StatisticalFunc),
    RelativeTime(f64, Unit, RelativeDirection),
    FunctionCall(String, Vec<Expression>), // Just in case we need it
    InterpolatedString(Vec<Expression>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelativeDirection {
    Ago,
    FromNow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalFunc {
    CountOf(String, Option<Box<Expression>>),
    AverageOf(String, Box<Expression>), // Value, Period
    MinOf(String),
    MaxOf(String),
    TrendOf(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(f64, Option<usize>),
    String(String),
    Quantity(f64, Unit, Option<usize>),
    TimeOfDay(String), // Simplification for now
    Date(String),
}
