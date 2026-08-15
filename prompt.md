I'm really interested in the concept of creating realistic feeling behaviour for AI in games. I've been facinated with Goal Oriented Action Planning as a technique for this and I have a hypothesis that if we compose different GOAP systems together we could created really real feeling agents for games that make complex and nuanced decisions for themselves.

The main issue I've faced so far is performance; the algorithm for decision making lends itself to the optimisations of things like A-star, but for abstract decisions a suitable heirestic isn't obvious. I'm wondering if layers of GOAP or sibling GOAP fields for different strains of reasoning might be best.

The core idea I'm working towards with outbound is a civilisation of agents who build a sociaty completely of their own volition based on their own desires and motives. The concept is really that to further their own aims they must work together. To do this I think we could potentially use various systems powered by GOAP reasoning, and my hope is that their composition provides interesting emergent behaviour.

The core GOAP framework should be designed as a seperate component and provide a clean interface for the implementation(s).

My working idea is to have villagers born with random distribution of desires and traits that effect their abilities. Those villagers can learn from eachother and develop skills. The better they are at skills the more likely they are to pick them to further their aims. They'll also be born with specific goals such as:

- To grow a family of 6 children.
- To amass wealth via commerce.
- To gain control over others.

Villagers with high building skill could teach other villagers to become builders, they would build houses for others. Blacksmiths could make tools to sell to builders. so on and so forth.

Ordinarily this would be very complex but I'm hoping we can produce this behaviour via the correct balance of GOAP systems with the right weighting of actions and goals.

I think we might need tiers. For example an overall goal of "have a child" could be broken down into hundreds of steps, which wouldn't scale with compute, so we might need more abstact goals at that level like "build a house" and "find a wife", which can have their own GOAP systems.

What do you think?


