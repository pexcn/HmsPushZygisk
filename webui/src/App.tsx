import React, { useEffect, useState, useMemo } from 'react';
import { getAppsData, writeConfig, AppInfo, ConfigMap } from './api';
import '@material/web/list/list.js';
import '@material/web/list/list-item.js';
import '@material/web/switch/switch.js';
import '@material/web/textfield/filled-text-field.js';
import '@material/web/progress/circular-progress.js';
import '@material/web/icon/icon.js';

function App() {
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [searchQuery, setSearchQuery] = useState<string>("");

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    const data = await getAppsData();
    setApps(data);
    setLoading(false);
  };

  const handleToggle = async (pkg: string, newValue: boolean) => {
    // Optimistic UI update
    setApps(currentApps => currentApps.map(app =>
      app.packageName === pkg ? { ...app, isEnabled: newValue } : app
    ));

    // Reconstruct the ConfigMap to save
    setApps(currentApps => {
      // Create new config map reflecting the change
      const newConfig: ConfigMap = new Map();
      const updatedApps = currentApps.map(app =>
        app.packageName === pkg ? { ...app, isEnabled: newValue } : app
      );

      updatedApps.forEach(app => {
        if (app.isEnabled) {
          newConfig.set(app.packageName, app.processes);
        }
      });

      // Write asynchronously
      writeConfig(newConfig);

      return updatedApps;
    });
  };

  const filteredApps = useMemo(() => {
    if (!searchQuery.trim()) return apps;
    const lowerQuery = searchQuery.toLowerCase();
    return apps.filter(app =>
      app.appName.toLowerCase().includes(lowerQuery) ||
      app.packageName.toLowerCase().includes(lowerQuery)
    );
  }, [apps, searchQuery]);

  return (
    <div className="app-container">

      <div className="search-container">
        <md-filled-text-field
          className="search-input"
          placeholder="Search apps..."
          value={searchQuery}
          onInput={(e: any) => setSearchQuery(e.target.value)}
        >
          <md-icon slot="leading-icon">search</md-icon>
        </md-filled-text-field>
      </div>

      {loading ? (
        <div className="loading">
          <md-circular-progress indeterminate className="spinner"></md-circular-progress>
          <div>Loading applications...</div>
        </div>
      ) : apps.length === 0 ? (
        <div className="empty">
          No configurable applications found.
        </div>
      ) : (
        <md-list className="app-list">
          {filteredApps.map((app) => (
            <md-list-item
              key={app.packageName}
              className="app-item"
              type="button"
              onClick={() => handleToggle(app.packageName, !app.isEnabled)}
            >
              <img
                slot="start"
                className="app-icon"
                src={app.iconUrl}
                alt={`${app.appName} icon`}
                onError={(e) => {
                  // Fallback if icon totally fails to load
                  (e.target as HTMLImageElement).src = 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" fill="transparent"><rect width="48" height="48" rx="10"/></svg>';
                }}
              />
              <div slot="headline" className="app-name" title={app.appName}>{app.appName}</div>
              <div slot="supporting-text" className="app-pkg" title={app.packageName}>{app.packageName}</div>

              <md-switch
                slot="end"
                selected={app.isEnabled}
                onClick={(e: React.MouseEvent<HTMLElement>) => e.stopPropagation()}
                onChange={(e: any) => handleToggle(app.packageName, e.target.selected)}
              ></md-switch>
            </md-list-item>
          ))}
        </md-list>
      )}
    </div>
  );
}

export default App;
