use ratatui::{style::Style, widgets::{Block, BorderType, Borders, Paragraph}};
use vtui::{events::*, input::*, prelude::*};
use vtui_core::layout::{NodeAxis, NodeConstraint, NodeFlex};

#[component]
fn Dashboard(c: Component) -> Node {
    c.draw(|canvas| {
        let widget = Paragraph::new("📊 Dashboard")
            .block(Block::default().borders(Borders::ALL).title("Main"));
        canvas.render_widget(canvas.area(), widget);
    });
    
    {
        let mut node = Node::from_component(c).ok().unwrap();
        node.add_static_child(Sidebar, ());
        node.add_static_child(MainContent, ());
        node.add_static_child(Footer, ());
        node.layout.axis = NodeAxis::Vertical;
        node.layout.constraint = NodeConstraint::Percentage(100);
        node
    }
}
#[component]
fn Sidebar(c: Component) -> Node {
    c.draw(|canvas| {
        let sidebar = Paragraph::new("📁 Files\n📈 Stats\n⚙️  Settings")
            .block(Block::default().borders(Borders::ALL).title("Sidebar"));
        canvas.render_widget(canvas.area(), sidebar);
    });
    
    {
        let mut node = Node::from_component(c).ok().unwrap();
        node.layout.constraint = NodeConstraint::Percentage(20);
        node
    }
}
#[component]
fn MainContent(c: Component) -> Node {
    c.draw(|canvas| {
        let content = Paragraph::new("🎯 Main Dashboard Content\n\n📈 Charts and graphs here...")
            .block(Block::default().borders(Borders::ALL).title("Content"));
        canvas.render_widget(canvas.area(), content);
    });
    
    {
        let mut node = Node::from_component(c).ok().unwrap();
        node.layout.constraint = NodeConstraint::Fill(1);
        node
    }
}
#[component]
fn Footer(c: Component) -> Node {
    c.draw(|canvas| {
        let footer = Paragraph::new("Status: Connected | Memory: 256MB | CPU: 12%")
            .block(Block::default().borders(Borders::ALL));
        canvas.render_widget(canvas.area(), footer);
    });
    
    {
        let mut node = Node::from_component(c).ok().unwrap();
        node.layout.constraint = NodeConstraint::Length(3);
        node
    }
}

fn main() {
    vtui::launch(Dashboard);
}
