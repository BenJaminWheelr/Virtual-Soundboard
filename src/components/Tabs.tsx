import type { TabId } from "../types";

type TabDefinition = {
  id: TabId;
  label: string;
};

const tabs: TabDefinition[] = [
  { id: "main", label: "Main" },
  { id: "soundboard", label: "Soundboard" },
  { id: "clips", label: "Audio Clips" },
  { id: "config", label: "Config" },
];

type TabsProps = {
  activeTab: TabId;
  onChange: (tab: TabId) => void;
};

function Tabs({ activeTab, onChange }: TabsProps) {
  return (
    <nav className="tabs" aria-label="Application tabs">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          className={tab.id === activeTab ? "tab active" : "tab"}
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}

export default Tabs;
