import { describe, expect, it } from "vitest"

import { mockNoteFolders } from "@/components/notes/notes.mock-data"
import {
  findFolderIdForNote,
  findNote,
} from "@/components/notes/notes.types"

describe("notes lookups", () => {
  it("finds a note nested in any folder", () => {
    expect(findNote(mockNoteFolders, "note-formatting")?.title).toBe(
      "Formatting reference",
    )
    expect(findNote(mockNoteFolders, "note-service")?.title).toBe(
      "curio-service migrations",
    )
  })

  it("returns nothing for an unselected or unknown note", () => {
    expect(findNote(mockNoteFolders, null)).toBeUndefined()
    expect(findNote(mockNoteFolders, "nope")).toBeUndefined()
    expect(findFolderIdForNote(mockNoteFolders, null)).toBeUndefined()
    expect(findFolderIdForNote(mockNoteFolders, "nope")).toBeUndefined()
  })

  it("resolves the folder holding a note so the sidebar can open it", () => {
    expect(findFolderIdForNote(mockNoteFolders, "note-roadmap")).toBe(
      "folder-product",
    )
  })

  it("keeps every note id unique across folders", () => {
    const ids = mockNoteFolders.flatMap((folder) =>
      folder.notes.map((note) => note.id),
    )

    expect(new Set(ids).size).toBe(ids.length)
  })

  it("stores note bodies as plate node arrays, never html strings", () => {
    for (const folder of mockNoteFolders) {
      for (const note of folder.notes) {
        expect(Array.isArray(note.body)).toBe(true)
        for (const node of note.body) {
          expect(typeof node).toBe("object")
          expect(node).toHaveProperty("children")
        }
      }
    }
  })
})
