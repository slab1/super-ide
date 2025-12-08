import { defineStore } from 'pinia'
import type { Settings } from '../types'

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    settings: {
      theme: 'dark' as const,
      fontSize: 14,
      fontFamily: 'Fira Code, Monaco, Consolas, monospace',
      tabSize: 2,
      wordWrap: true,
      minimap: true,
      lineNumbers: true,
      autoSave: true
    } as Settings,
    isLoaded: false
  }),

  actions: {
    loadSettings() {
      if (this.isLoaded) return
      
      try {
        const saved = localStorage.getItem('super-ide-settings')
        if (saved) {
          const parsed = JSON.parse(saved)
          this.settings = { ...this.settings, ...parsed }
        }
        this.isLoaded = true
      } catch (error) {
        console.error('Failed to load settings:', error)
        this.isLoaded = true
      }
    },

    saveSettings() {
      try {
        localStorage.setItem('super-ide-settings', JSON.stringify(this.settings))
      } catch (error) {
        console.error('Failed to save settings:', error)
      }
    },

    updateSettings(newSettings: Partial<Settings>) {
      this.settings = { ...this.settings, ...newSettings }
      this.saveSettings()
    },

    setTheme(theme: 'dark' | 'light') {
      this.settings.theme = theme
      this.saveSettings()
      
      // Apply theme to document
      document.documentElement.setAttribute('data-theme', theme)
      document.body.className = theme === 'dark' ? 'dark' : 'light'
    },

    setFontSize(size: number) {
      this.settings.fontSize = Math.max(8, Math.min(72, size))
      this.saveSettings()
    },

    setFontFamily(fontFamily: string) {
      this.settings.fontFamily = fontFamily
      this.saveSettings()
    },

    setTabSize(size: number) {
      this.settings.tabSize = Math.max(1, Math.min(8, size))
      this.saveSettings()
    },

    setWordWrap(enabled: boolean) {
      this.settings.wordWrap = enabled
      this.saveSettings()
    },

    setMinimap(enabled: boolean) {
      this.settings.minimap = enabled
      this.saveSettings()
    },

    setLineNumbers(enabled: boolean) {
      this.settings.lineNumbers = enabled
      this.saveSettings()
    },

    setAutoSave(enabled: boolean) {
      this.settings.autoSave = enabled
      this.saveSettings()
    },

    resetToDefaults() {
      this.settings = {
        theme: 'dark',
        fontSize: 14,
        fontFamily: 'Fira Code, Monaco, Consolas, monospace',
        tabSize: 2,
        wordWrap: true,
        minimap: true,
        lineNumbers: true,
        autoSave: true
      }
      this.saveSettings()
    },

    exportSettings(): string {
      return JSON.stringify(this.settings, null, 2)
    },

    importSettings(settingsJson: string) {
      try {
        const parsed = JSON.parse(settingsJson)
        this.updateSettings(parsed)
      } catch (error) {
        throw new Error('Invalid settings JSON')
      }
    }
  }
})