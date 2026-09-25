import { useMemo, useRef, useState } from "react";

import { PageHeader } from "@/components/page-header";
import { VaultCategoryFilter } from "@/components/vault/vault-category-filter";
import {
  VAULT_ITEMS_PER_PAGE,
  VAULT_MOCK_DATA,
} from "@/components/vault/vault.mock-data";
import { VaultPagination } from "@/components/vault/vault-pagination";
import { VaultResultsList } from "@/components/vault/vault-results-list";
import { VaultSearchBar } from "@/components/vault/vault-search-bar";

const ALL_CATEGORIES = "All";

const Vault = () => {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState(ALL_CATEGORIES);
  const [currentPage, setCurrentPage] = useState(1);
  const scrollRef = useRef<HTMLDivElement>(null);

  const categories = useMemo(() => {
    const counts = new Map<string, number>();
    for (const item of VAULT_MOCK_DATA) {
      counts.set(item.category, (counts.get(item.category) ?? 0) + 1);
    }
    return [
      { name: ALL_CATEGORIES, count: VAULT_MOCK_DATA.length },
      ...Array.from(counts, ([name, count]) => ({ name, count })),
    ];
  }, []);

  const filteredData = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();

    return VAULT_MOCK_DATA.filter((item) => {
      const matchesSearch =
        normalizedQuery.length === 0 ||
        item.question.toLowerCase().includes(normalizedQuery) ||
        item.answer.toLowerCase().includes(normalizedQuery);
      const matchesCategory =
        selectedCategory === ALL_CATEGORIES || item.category === selectedCategory;

      return matchesSearch && matchesCategory;
    });
  }, [searchQuery, selectedCategory]);

  const totalPages = Math.max(
    1,
    Math.ceil(filteredData.length / VAULT_ITEMS_PER_PAGE),
  );
  const safeCurrentPage = Math.min(currentPage, totalPages);
  const startIndex = (safeCurrentPage - 1) * VAULT_ITEMS_PER_PAGE;
  const endIndex = startIndex + VAULT_ITEMS_PER_PAGE;
  const paginatedData = filteredData.slice(startIndex, endIndex);

  const handleSearchChange = (value: string) => {
    setSearchQuery(value);
    setCurrentPage(1);
  };

  const handleCategoryChange = (category: string) => {
    setSelectedCategory(category);
    setCurrentPage(1);
  };

  const handlePageChange = (page: number) => {
    setCurrentPage(Math.min(Math.max(page, 1), totalPages));
    scrollRef.current?.scrollTo({ top: 0 });
  };

  const clearFilters = () => {
    setSearchQuery("");
    setSelectedCategory(ALL_CATEGORIES);
    setCurrentPage(1);
  };

  const resultLabel = `${filteredData.length} ${
    filteredData.length === 1 ? "entry" : "entries"
  }`;

  return (
    <>
      <PageHeader title="Vault" meta={`${VAULT_MOCK_DATA.length} saved answers`} />

      <div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto w-full max-w-[72rem] px-5 pb-16 pt-8 sm:px-8 lg:px-12 lg:pt-10">
          <section aria-label="Search the vault" className="rise-in space-y-4">
            <p className="max-w-2xl font-display text-xl italic leading-snug text-muted-foreground">
              Every answer you chose to keep, searchable in one place.
            </p>
            <VaultSearchBar
              value={searchQuery}
              onValueChange={handleSearchChange}
            />
            <VaultCategoryFilter
              categories={categories}
              selectedCategory={selectedCategory}
              onCategoryChange={handleCategoryChange}
            />
          </section>

          <p
            aria-live="polite"
            className="eyebrow mt-10 mb-3 text-muted-foreground"
          >
            {resultLabel}
            {totalPages > 1 ? ` · page ${safeCurrentPage} of ${totalPages}` : null}
          </p>

          <div className="rise-in [--rise-index:1]">
            <VaultResultsList
              items={paginatedData}
              query={searchQuery}
              onClearFilters={clearFilters}
            />
          </div>

          <VaultPagination
            currentPage={safeCurrentPage}
            totalPages={totalPages}
            totalItems={filteredData.length}
            startIndex={startIndex}
            endIndex={endIndex}
            onPageChange={handlePageChange}
          />
        </div>
      </div>
    </>
  );
};

export default Vault;
